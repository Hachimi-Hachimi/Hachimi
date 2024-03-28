mod Localize;
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
pub mod MasterSkillData;
mod PartsSingleModeSkillLearningListItem;
mod MasterMissionData;

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
    MasterSkillData::init(image);
    PartsSingleModeSkillLearningListItem::init(image);
    MasterMissionData::init(image);
}