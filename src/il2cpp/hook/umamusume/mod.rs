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
}