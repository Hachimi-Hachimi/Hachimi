pub mod Localize;
pub mod TextId;
pub mod StoryRaceTextAsset;
mod LyricsController;
pub mod StoryTimelineData;
pub mod StoryTimelineBlockData;
pub mod StoryTimelineTrackData;
pub mod StoryTimelineTextClipData;
pub mod GallopUtil;
mod UIManager;
pub mod GraphicSettings;
mod CameraController;
pub mod SingleModeStartResultCharaViewer;
pub mod WebViewManager;
pub mod DialogCommon;
mod PartsSingleModeSkillLearningListItem;
mod MasterMissionData;
mod TrainingParamChangeA2U;
pub mod WebViewDefine;
pub mod TextFrame;
mod PartsSingleModeSkillListItem;
pub mod FlashActionPlayer;
pub mod TextRubyData;
pub mod TextDotData;
pub mod GameSystem;
mod StoryViewTextControllerLandscape;
mod StoryViewTextControllerSingleMode;
mod JikkyoDisplay;
pub mod Screen;
mod TrainingParamChangePlate;
mod SingleModeUtils;
mod MasterSingleModeTurn;
mod TextFontManager;
mod TextFormat;
mod TextCommon;
mod TextMeshProUguiCommon;
mod StoryChoiceController;
mod StoryViewController;
mod StoryTimelineClipData;
mod StoryTimelineCharaTrackData;
mod CharacterNoteTopView;
mod CharacterNoteTopViewController;
mod ViewControllerBase;
mod ButtonCommon;
mod NowLoading;
pub mod StoryTimelineController;
mod DialogRaceOrientation;
mod RaceInfo;
mod RaceUtil;
mod SaveDataManager;
mod ApplicationSettingSaveLoader;
mod LiveTheaterCharaSelect;
mod LiveTheaterViewController;
pub mod CySpringController;

#[cfg(target_os = "windows")]
pub mod SceneManager;

#[cfg(target_os = "windows")]
mod PaymentUtility;

pub fn init() {
    get_assembly_image_or_return!(image, "umamusume.dll");

    Localize::init(image);
    TextId::init(image);
    StoryRaceTextAsset::init(image);
    LyricsController::init(image);
    StoryTimelineData::init(image);
    StoryTimelineBlockData::init(image);
    StoryTimelineTrackData::init(image);
    StoryTimelineTextClipData::init(image);
    GallopUtil::init(image);
    UIManager::init(image);
    GraphicSettings::init(image);
    CameraController::init(image);
    SingleModeStartResultCharaViewer::init(image);
    WebViewManager::init(image);
    DialogCommon::init(image);
    PartsSingleModeSkillLearningListItem::init(image);
    MasterMissionData::init(image);
    TrainingParamChangeA2U::init(image);
    TextFrame::init(image);
    PartsSingleModeSkillListItem::init(image);
    FlashActionPlayer::init(image);
    TextRubyData::init(image);
    TextDotData::init(image);
    GameSystem::init(image);
    StoryViewTextControllerLandscape::init(image);
    StoryViewTextControllerSingleMode::init(image);
    JikkyoDisplay::init(image);
    Screen::init(image);
    TrainingParamChangePlate::init(image);
    SingleModeUtils::init(image);
    MasterSingleModeTurn::init(image);
    TextFontManager::init(image);
    TextFormat::init(image);
    TextCommon::init(image);
    TextMeshProUguiCommon::init(image);
    StoryChoiceController::init(image);
    StoryViewController::init(image);
    StoryTimelineClipData::init(image);
    StoryTimelineCharaTrackData::init(image);
    CharacterNoteTopView::init(image);
    CharacterNoteTopViewController::init(image);
    ViewControllerBase::init(image);
    ButtonCommon::init(image);
    NowLoading::init(image);
    StoryTimelineController::init(image);
    DialogRaceOrientation::init(image);
    RaceInfo::init(image);
    RaceUtil::init(image);
    SaveDataManager::init(image);
    ApplicationSettingSaveLoader::init(image);
    LiveTheaterCharaSelect::init(image);
    LiveTheaterViewController::init(image);
    CySpringController::init(image);

    #[cfg(target_os = "windows")]
    {
        SceneManager::init(image);
        PaymentUtility::init(image);
    }
}