use windows::{core::HRESULT, Win32::Graphics::{
    Direct3D11::{
        ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
        D3D11_RENDER_TARGET_VIEW_DESC, D3D11_RTV_DIMENSION_TEXTURE2D
    },
    Dxgi::{Common::DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, IDXGISwapChain}
}};

use crate::core::Error;

use super::d3d11_backup;

pub struct D3D11Painter {
    renderer: egui_directx11::Renderer,
    swap_chain: IDXGISwapChain,
    render_target: Option<ID3D11RenderTargetView>,
    backup_state: d3d11_backup::BackupState
}

impl D3D11Painter {
    pub fn new(swap_chain: IDXGISwapChain) -> Result<D3D11Painter, Error> {
        let device: ID3D11Device = unsafe { swap_chain.GetDevice() }.map_err(|_|
            Error::RuntimeError("Failed to get D3D11 device".to_owned())
        )?;
        let renderer = egui_directx11::Renderer::new(&device).map_err(|e|
            Error::RuntimeError(e.to_string())
        )?;

        let mut painter = D3D11Painter {
            renderer,
            swap_chain,
            render_target: None,
            backup_state: Default::default()
        };
        painter.init_render_target();

        Ok(painter)
    }

    pub fn swap_chain(&self) -> &IDXGISwapChain {
        &self.swap_chain
    }

    fn get_device(&self) -> Result<ID3D11Device, Error> {
        unsafe { self.swap_chain.GetDevice() }.map_err(|_|
            Error::RuntimeError("Failed to get D3D11 device".to_owned())
        )
    }

    fn get_device_context(device: &ID3D11Device) -> Result<ID3D11DeviceContext, Error> {
        unsafe { device.GetImmediateContext() }.map_err(|_|
            Error::RuntimeError("Failed to get D3D11 device context".to_owned())
        )
    }

    /// Call this in the ResizeBuffers hook, with orig_fn calling the original function
    pub fn resize_buffers(&mut self, orig_fn: impl FnOnce() -> HRESULT) -> HRESULT {
        self.render_target = None;
        
        // Has to be called after dropping the render target!
        let res = orig_fn();
        if res.is_ok() {
            self.init_render_target();
        }

        res
    }

    fn init_render_target(&mut self) {
        let Ok(backbuffer) = (unsafe { self.swap_chain.GetBuffer::<ID3D11Texture2D>(0) }) else {
            error!("Failed to get swapchain's backbuffer");
            return;
        };

        let Ok(device) = self.get_device() else {
            error!("Failed to get D3D11 device");
            return;
        };

        let mut render_target_desc = D3D11_RENDER_TARGET_VIEW_DESC::default();
        render_target_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM_SRGB;
        render_target_desc.ViewDimension = D3D11_RTV_DIMENSION_TEXTURE2D;
        if let Err(e) = unsafe { device.CreateRenderTargetView(
            &backbuffer,
            Some(&render_target_desc),
            Some(&mut self.render_target)
        )} {
            error!("Failed to create render target view: {}", e);
            return;
        };
    }

    /// Call this in the Present hook, before calling the orig fn
    pub fn present(
        &mut self, egui_ctx: &egui::Context, egui_output: egui_directx11::RendererOutput, scale_factor: f32
    ) -> Result<(), Error> {
        let Some(render_target) = &self.render_target else {
            return Ok(());
        };

        let device_context = Self::get_device_context(&self.get_device()?)?;
        unsafe { self.backup_state.save(&device_context); }

        if let Err(e) = self.renderer.render(&device_context, render_target, egui_ctx, egui_output, scale_factor) {
            return Err(Error::RuntimeError(e.to_string()));
        }

        unsafe { self.backup_state.restore(&device_context); }
        Ok(())
    }
}