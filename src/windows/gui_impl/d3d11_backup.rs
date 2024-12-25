use std::mem::zeroed;
use windows::Win32::{
    Foundation::RECT,
    Graphics::{
        Direct3D::D3D_PRIMITIVE_TOPOLOGY,
        Direct3D11::{
            ID3D11BlendState, ID3D11Buffer, ID3D11ClassInstance, ID3D11DepthStencilState,
            ID3D11DeviceContext, ID3D11GeometryShader, ID3D11InputLayout, ID3D11PixelShader,
            ID3D11RasterizerState, ID3D11SamplerState, ID3D11ShaderResourceView,
            ID3D11VertexShader, D3D11_VIEWPORT,
            D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE,
        },
        Dxgi::Common::DXGI_FORMAT,
    },
};

#[derive(Default)]
pub struct BackupState {
    scissor_rects: [RECT; D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE as _],
    scissor_count: u32,

    viewports: [D3D11_VIEWPORT; D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE as _],
    viewport_count: u32,

    raster_state: Option<ID3D11RasterizerState>,

    blend_state: Option<ID3D11BlendState>,
    blend_factor: [f32; 4],
    blend_mask: u32,

    depth_stencil_state: Option<ID3D11DepthStencilState>,
    stencil_ref: u32,

    pixel_shader_resource: Option<ID3D11ShaderResourceView>,

    sampler: Option<ID3D11SamplerState>,

    vertex_shader: Option<ID3D11VertexShader>,
    vertex_shader_instances: ClassInstances,
    vertex_shader_instances_count: u32,

    geometry_shader: Option<ID3D11GeometryShader>,
    geometry_shader_instances: ClassInstances,
    geometry_shader_instances_count: u32,

    pixel_shader: Option<ID3D11PixelShader>,
    pixel_shader_instances: ClassInstances,
    pixel_shader_instances_count: u32,

    constant_buffer: Option<ID3D11Buffer>,
    primitive_topology: D3D_PRIMITIVE_TOPOLOGY,

    index_buffer: Option<ID3D11Buffer>,
    index_buffer_format: DXGI_FORMAT,
    index_buffer_offest: u32,

    vertex_buffer: Option<ID3D11Buffer>,
    vertex_buffer_strides: u32,
    vertex_buffer_offsets: u32,

    input_layout: Option<ID3D11InputLayout>,
}

impl BackupState {
    #[inline]
    pub unsafe fn save(&mut self, ctx: &ID3D11DeviceContext) {
        // Rasterizer parameters
        self.scissor_count = D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE;
        self.viewport_count = D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE;
        ctx.RSGetScissorRects(&mut self.scissor_count, Some(self.scissor_rects.as_mut_ptr()));
        ctx.RSGetViewports(&mut self.viewport_count, Some(self.viewports.as_mut_ptr()));
        self.raster_state = ctx.RSGetState().ok();

        // Output-Merger parameters
        ctx.OMGetBlendState(
            Some(&mut self.blend_state),
            Some(&mut self.blend_factor),
            Some(&mut self.blend_mask),
        );
        ctx.OMGetDepthStencilState(Some(&mut self.depth_stencil_state), Some(&mut self.stencil_ref));

        // Pixel Shader parameters
        let mut pixel_shader_resources = [None];
        ctx.PSGetShaderResources(0, Some(&mut pixel_shader_resources));
        self.pixel_shader_resource = pixel_shader_resources[0].take();

        let mut samplers = [None];
        ctx.PSGetSamplers(0, Some(&mut samplers));
        self.sampler = samplers[0].take();

        // Shaders
        self.pixel_shader_instances_count = 256;
        self.vertex_shader_instances_count = 256;
        self.geometry_shader_instances_count = 256;

        ctx.PSGetShader(
            &mut self.pixel_shader,
            Some(self.pixel_shader_instances.as_mut_ptr()),
            Some(&mut self.pixel_shader_instances_count),
        );
        ctx.VSGetShader(
            &mut self.vertex_shader,
            Some(self.vertex_shader_instances.as_mut_ptr()),
            Some(&mut self.vertex_shader_instances_count),
        );
        ctx.GSGetShader(
            &mut self.geometry_shader,
            Some(self.geometry_shader_instances.as_mut_ptr()),
            Some(&mut self.geometry_shader_instances_count),
        );

        // Vertex Shader parameters
        let mut constant_buffers = [None];
        ctx.VSGetConstantBuffers(0, Some(&mut constant_buffers));
        self.constant_buffer = constant_buffers[0].take();

        // Input-Assembler parameters
        self.primitive_topology = ctx.IAGetPrimitiveTopology();
        ctx.IAGetIndexBuffer(
            Some(&mut self.index_buffer),
            Some(&mut self.index_buffer_format),
            Some(&mut self.index_buffer_offest),
        );
        ctx.IAGetVertexBuffers(
            0,
            1,
            Some(&mut self.vertex_buffer),
            Some(&mut self.vertex_buffer_strides),
            Some(&mut self.vertex_buffer_offsets),
        );
        self.input_layout = ctx.IAGetInputLayout().ok();
    }

    #[inline]
    pub unsafe fn restore(&mut self, ctx: &ID3D11DeviceContext) {
        // Rasterizer parameters
        ctx.RSSetScissorRects(Some(&self.scissor_rects[..self.scissor_count as usize]));
        ctx.RSSetViewports(Some(&self.viewports[..self.viewport_count as usize]));
        if let Some(raster_state) = self.raster_state.take() {
            ctx.RSSetState(&raster_state);
        }

        // Output-Merger parameters
        if let Some(blend_state) = self.blend_state.take() {
            ctx.OMSetBlendState(
                &blend_state,
                Some(&self.blend_factor),
                self.blend_mask,
            );
        }
        if let Some(depth_stencil_state) = self.depth_stencil_state.take() {
            ctx.OMSetDepthStencilState(&depth_stencil_state, self.stencil_ref);
        }

        // Pixel Shader parameters
        ctx.PSSetShaderResources(0, Some(&[self.pixel_shader_resource.take()]));
        ctx.PSSetSamplers(0, Some(&[self.sampler.take()]));

        // Shaders
        if let Some(pixel_shader) = self.pixel_shader.take() {
            ctx.PSSetShader(
                &pixel_shader,
                Some(&self.pixel_shader_instances.0[..self.pixel_shader_instances_count as usize])
            );
        }
        self.pixel_shader_instances.release();

        if let Some(vertex_shader) = self.vertex_shader.take() {
            ctx.VSSetShader(
                &vertex_shader,
                Some(&self.vertex_shader_instances.0[..self.vertex_shader_instances_count as usize])
            );
        }
        self.vertex_shader_instances.release();

        if let Some(geometry_shader) = self.geometry_shader.take() {
            ctx.GSSetShader(
                &geometry_shader,
                Some(&self.geometry_shader_instances.0[..self.geometry_shader_instances_count as usize])
            );
        }
        self.geometry_shader_instances.release();

        // Vertex Shader parameters
        ctx.VSSetConstantBuffers(0, Some(&[self.constant_buffer.take()]));

        // Input-Assembler parameters
        ctx.IASetPrimitiveTopology(self.primitive_topology);
        if let Some(index_buffer) = self.index_buffer.take() {
            ctx.IASetIndexBuffer(
                &index_buffer,
                self.index_buffer_format,
                self.index_buffer_offest,
            );
        }
        ctx.IASetVertexBuffers(
            0,
            1,
            Some(&self.vertex_buffer.take()),
            Some(&self.vertex_buffer_strides),
            Some(&self.vertex_buffer_offsets),
        );
        if let Some(input_layout) = self.input_layout.take() {
            ctx.IASetInputLayout(&input_layout);
        }
    }
}

struct ClassInstances([Option<ID3D11ClassInstance>; 256]);

impl ClassInstances {
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut Option<ID3D11ClassInstance> {
        &mut self.0[0]
    }

    #[inline]
    pub fn release(&mut self) {
        self.0.iter().for_each(drop);
    }
}

impl Default for ClassInstances {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}