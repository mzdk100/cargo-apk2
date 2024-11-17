use crate::error::NdkError;
use serde::{Deserialize, Serialize, Serializer};
use std::{fs::File, io::Write, path::Path};

/// Android [manifest 元素](https://developer.android.com/guide/topics/manifest/manifest-element), containing an [`Application`] element.
// quick_xml规定#[serde(rename)]的值如果带有`@`符号表示属性，否则表示tag
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "manifest")]
pub struct AndroidManifest {
    #[serde(rename(serialize = "@xmlns:android"))]
    #[serde(default = "default_namespace")]
    ns_android: String,
    #[serde(default, rename(serialize = "@package"))]
    pub package: String,
    #[serde(
        rename(serialize = "@android:sharedUserId"),
        skip_serializing_if = "Option::is_none"
    )]
    pub shared_user_id: Option<String>,
    #[serde(
        rename(serialize = "@android:versionCode"),
        skip_serializing_if = "Option::is_none"
    )]
    pub version_code: Option<u32>,
    #[serde(
        rename(serialize = "@android:versionName"),
        skip_serializing_if = "Option::is_none"
    )]
    pub version_name: Option<String>,

    #[serde(rename(serialize = "uses-sdk"))]
    #[serde(default)]
    pub sdk: Sdk,

    #[serde(rename(serialize = "uses-feature"))]
    #[serde(default)]
    pub uses_feature: Vec<Feature>,

    #[serde(rename(serialize = "uses-permission"))]
    #[serde(default)]
    pub uses_permission: Vec<Permission>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queries: Option<Queries>,

    #[serde(default)]
    pub application: Application,
}

impl Default for AndroidManifest {
    fn default() -> Self {
        Self {
            ns_android: default_namespace(),
            package: Default::default(),
            shared_user_id: Default::default(),
            version_code: Default::default(),
            version_name: Default::default(),
            sdk: Default::default(),
            uses_feature: Default::default(),
            uses_permission: Default::default(),
            queries: Default::default(),
            application: Default::default(),
        }
    }
}

impl AndroidManifest {
    pub fn write_to(&self, dir: &Path) -> Result<(), NdkError> {
        let mut buf = String::new();
        quick_xml::se::to_writer(&mut buf, &self).map_err(|e| NdkError::Serialize(e))?;
        let mut file = File::create(dir.join("AndroidManifest.xml"))?;
        file.write_all(buf.as_bytes())?;
        Ok(())
    }
}

/// Android [application 元素](https://developer.android.com/guide/topics/manifest/application-element), containing an [`Activity`] element.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Application {
    #[serde(
        rename(serialize = "@android:debuggable"),
        skip_serializing_if = "Option::is_none"
    )]
    pub debuggable: Option<bool>,
    #[serde(
        rename(serialize = "@android:theme"),
        skip_serializing_if = "Option::is_none"
    )]
    pub theme: Option<String>,
    #[serde(rename(serialize = "@android:hasCode"))]
    #[serde(default)]
    pub has_code: bool,
    #[serde(
        rename(serialize = "@android:icon"),
        skip_serializing_if = "Option::is_none"
    )]
    pub icon: Option<String>,
    #[serde(rename(serialize = "@android:label"))]
    #[serde(default)]
    pub label: String,
    #[serde(
        rename(serialize = "@android:extractNativeLibs"),
        skip_serializing_if = "Option::is_none"
    )]
    pub extract_native_libs: Option<bool>,
    #[serde(
        rename(serialize = "@android:usesCleartextTraffic"),
        skip_serializing_if = "Option::is_none"
    )]
    pub uses_cleartext_traffic: Option<bool>,

    #[serde(rename(serialize = "meta-data"))]
    #[serde(default)]
    pub meta_data: Vec<MetaData>,
    #[serde(default)]
    pub activity: Activity,
}

/// Android [activity 元素](https://developer.android.com/guide/topics/manifest/activity-element).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Activity {
    #[serde(rename(serialize = "@android:configChanges"))]
    #[serde(
        default = "default_config_changes",
        skip_serializing_if = "Option::is_none"
    )]
    pub config_changes: Option<String>,
    #[serde(
        rename(serialize = "@android:label"),
        skip_serializing_if = "Option::is_none"
    )]
    pub label: Option<String>,
    #[serde(
        rename(serialize = "@android:launchMode"),
        skip_serializing_if = "Option::is_none"
    )]
    pub launch_mode: Option<String>,
    #[serde(rename(serialize = "@android:name"))]
    #[serde(default = "default_activity_name")]
    pub name: String,
    #[serde(
        rename(serialize = "@android:screenOrientation"),
        skip_serializing_if = "Option::is_none"
    )]
    pub orientation: Option<String>,
    #[serde(
        rename(serialize = "@android:exported"),
        skip_serializing_if = "Option::is_none"
    )]
    pub exported: Option<bool>,
    #[serde(
        rename(serialize = "@android:resizeableActivity"),
        skip_serializing_if = "Option::is_none"
    )]
    pub resizeable_activity: Option<bool>,
    #[serde(
        rename(serialize = "@android:alwaysRetainTaskState"),
        skip_serializing_if = "Option::is_none"
    )]
    pub always_retain_task_state: Option<bool>,

    #[serde(rename(serialize = "meta-data"))]
    #[serde(default)]
    pub meta_data: Vec<MetaData>,
    /// 如果任何意图过滤器中都不存在“MAIN”动作，则默认的“MAIN”过滤器由“cargo-apk2”序列化。
    #[serde(rename(serialize = "intent-filter"))]
    #[serde(default)]
    pub intent_filter: Vec<IntentFilter>,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            config_changes: default_config_changes(),
            label: None,
            launch_mode: None,
            name: default_activity_name(),
            orientation: None,
            exported: None,
            resizeable_activity: None,
            always_retain_task_state: None,
            meta_data: Default::default(),
            intent_filter: Default::default(),
        }
    }
}

/// Android [intent filter element](https://developer.android.com/guide/topics/manifest/intent-filter-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IntentFilter {
    /// 序列化包裹在 `<action android:name="..." />` 中的字符串。
    #[serde(serialize_with = "serialize_actions")]
    #[serde(rename(serialize = "action"))]
    #[serde(default)]
    pub actions: Vec<String>,
    /// 序列化为结构向量以实现正确的 xml 格式
    #[serde(serialize_with = "serialize_categories")]
    #[serde(rename(serialize = "category"))]
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub data: Vec<IntentFilterData>,
}

fn serialize_actions<S>(actions: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;

    #[derive(Serialize)]
    struct Action {
        #[serde(rename = "@android:name")]
        name: String,
    }
    let mut seq = serializer.serialize_seq(Some(actions.len()))?;
    for action in actions {
        seq.serialize_element(&Action {
            name: action.clone(),
        })?;
    }
    seq.end()
}

fn serialize_categories<S>(categories: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;

    #[derive(Serialize)]
    struct Category {
        #[serde(rename = "@android:name")]
        pub name: String,
    }

    let mut seq = serializer.serialize_seq(Some(categories.len()))?;
    for category in categories {
        seq.serialize_element(&Category {
            name: category.clone(),
        })?;
    }
    seq.end()
}

/// Android [intent filter data 元素](https://developer.android.com/guide/topics/manifest/data-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IntentFilterData {
    #[serde(rename(serialize = "@android:scheme"))]
    pub scheme: Option<String>,
    #[serde(rename(serialize = "@android:host"))]
    pub host: Option<String>,
    #[serde(rename(serialize = "@android:port"))]
    pub port: Option<String>,
    #[serde(rename(serialize = "@android:path"))]
    pub path: Option<String>,
    #[serde(rename(serialize = "@android:pathPattern"))]
    pub path_pattern: Option<String>,
    #[serde(rename(serialize = "@android:pathPrefix"))]
    pub path_prefix: Option<String>,
    #[serde(rename(serialize = "@android:mimeType"))]
    pub mime_type: Option<String>,
}

/// Android [meta-data 元素](https://developer.android.com/guide/topics/manifest/meta-data-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MetaData {
    #[serde(rename(serialize = "@android:name"))]
    pub name: String,
    #[serde(rename(serialize = "@android:value"))]
    pub value: String,
}

//noinspection SpellCheckingInspection
/// Android [uses-feature 元素](https://developer.android.com/guide/topics/manifest/uses-feature-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Feature {
    #[serde(
        rename(serialize = "@android:name"),
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,
    #[serde(
        rename(serialize = "@android:required"),
        skip_serializing_if = "Option::is_none"
    )]
    pub required: Option<bool>,
    /// `version` 字段当前用于以下功能：
    ///
    /// - `name="android.hardware.vulkan.compute"`: 所需的最低计算功能级别。请参阅 [Android 文档](https://developer.android.com/reference/android/content/pm/PackageManager#FEATURE_VULKAN_HARDWARE_COMPUTE)，了解可用级别以及所需/提供的相应 Vulkan 功能。
    /// - `name="android.hardware.vulkan.level"`: Vulkan 的最低要求。请参阅 [Android 文档](https://developer.android.com/reference/android/content/pm/PackageManager#FEATURE_VULKAN_HARDWARE_LEVEL)了解可用级别以及所需/提供的相应 Vulkan 功能。
    /// - `name="android.hardware.vulkan.version"`: 表示 Vulkan 的 `VkPhysicalDeviceProperties::apiVersion` 的值。请参阅 [Android 文档](https://developer.android.com/reference/android/content/pm/PackageManager#FEATURE_VULKAN_HARDWARE_VERSION)以了解可用级别以及所需/提供的相应 Vulkan 功能。
    #[serde(
        rename(serialize = "@android:version"),
        skip_serializing_if = "Option::is_none"
    )]
    pub version: Option<u32>,
    #[serde(
        rename(serialize = "@android:glEsVersion"),
        skip_serializing_if = "Option::is_none"
    )]
    #[serde(serialize_with = "serialize_opengles_version")]
    pub opengles_version: Option<(u8, u8)>,
}

//noinspection SpellCheckingInspection
fn serialize_opengles_version<S>(
    version: &Option<(u8, u8)>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match version {
        Some(version) => {
            let opengles_version = format!("0x{:04}{:04}", version.0, version.1);
            serializer.serialize_some(&opengles_version)
        }
        None => serializer.serialize_none(),
    }
}

/// Android [uses-permission 元素](https://developer.android.com/guide/topics/manifest/uses-permission-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Permission {
    #[serde(rename(serialize = "@android:name"))]
    pub name: String,
    #[serde(
        rename(serialize = "@android:maxSdkVersion"),
        skip_serializing_if = "Option::is_none"
    )]
    pub max_sdk_version: Option<u32>,
}

/// Android [package 元素](https://developer.android.com/guide/topics/manifest/queries-element#package).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Package {
    #[serde(rename(serialize = "@android:name"))]
    pub name: String,
}

/// Android [provider 元素](https://developer.android.com/guide/topics/manifest/queries-element#provider).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct QueryProvider {
    #[serde(rename(serialize = "@android:authorities"))]
    pub authorities: String,

    // 规范规定，对于包含在“queries”元素中的提供程序，仅需要一个“authorities”属性，但这对于 aapt 支持是必需的，并且当 cargo-apk2 迁移到 aapt2 时，应将其设为可选
    #[serde(rename(serialize = "@android:name"))]
    pub name: String,
}

/// Android [queries 元素](https://developer.android.com/guide/topics/manifest/queries-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Queries {
    #[serde(default)]
    pub package: Vec<Package>,
    #[serde(default)]
    pub intent: Vec<IntentFilter>,
    #[serde(default)]
    pub provider: Vec<QueryProvider>,
}

/// Android [uses-sdk 元素](https://developer.android.com/guide/topics/manifest/uses-sdk-element)。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sdk {
    #[serde(
        rename(serialize = "@android:minSdkVersion"),
        skip_serializing_if = "Option::is_none"
    )]
    pub min_sdk_version: Option<u32>,
    #[serde(
        rename(serialize = "@android:targetSdkVersion"),
        skip_serializing_if = "Option::is_none"
    )]
    pub target_sdk_version: Option<u32>,
    #[serde(
        rename(serialize = "@android:maxSdkVersion"),
        skip_serializing_if = "Option::is_none"
    )]
    pub max_sdk_version: Option<u32>,
}

impl Default for Sdk {
    fn default() -> Self {
        Self {
            min_sdk_version: Some(24),
            target_sdk_version: None,
            max_sdk_version: None,
        }
    }
}

//noinspection HttpUrlsUsage
fn default_namespace() -> String {
    "http://schemas.android.com/apk/res/android".to_string()
}

fn default_activity_name() -> String {
    "android.app.NativeActivity".to_string()
}

fn default_config_changes() -> Option<String> {
    Some("orientation|keyboardHidden|screenSize".to_string())
}
