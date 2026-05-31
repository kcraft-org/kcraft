use std::any::Any;

pub trait SettingValue: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn SettingValue>;
    fn to_value_string(&self) -> String;
}

impl SettingValue for String {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn SettingValue> {
        Box::new(self.clone())
    }
    fn to_value_string(&self) -> String {
        self.clone()
    }
}

impl SettingValue for bool {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn SettingValue> {
        Box::new(*self)
    }
    fn to_value_string(&self) -> String {
        self.to_string()
    }
}

impl SettingValue for i32 {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn SettingValue> {
        Box::new(*self)
    }
    fn to_value_string(&self) -> String {
        self.to_string()
    }
}

impl SettingValue for i64 {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn SettingValue> {
        Box::new(*self)
    }
    fn to_value_string(&self) -> String {
        self.to_string()
    }
}

impl SettingValue for f64 {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn SettingValue> {
        Box::new(*self)
    }
    fn to_value_string(&self) -> String {
        self.to_string()
    }
}

pub struct Setting {
    synonyms: Vec<String>,
    default_value: Box<dyn SettingValue>,
    current_value: Option<Box<dyn SettingValue>>,
}

impl Clone for Setting {
    fn clone(&self) -> Self {
        Setting {
            synonyms: self.synonyms.clone(),
            default_value: self.default_value.clone_box(),
            current_value: self.current_value.as_ref().map(|v| v.clone_box()),
        }
    }
}

impl Setting {
    pub fn new<T: SettingValue + 'static>(synonyms: Vec<String>, default_value: T) -> Self {
        Setting {
            synonyms,
            default_value: Box::new(default_value),
            current_value: None,
        }
    }

    pub fn id(&self) -> &str {
        self.synonyms.first().map(|s| s.as_str()).unwrap_or("")
    }

    pub fn config_keys(&self) -> &[String] {
        &self.synonyms
    }

    pub fn default_value(&self) -> &dyn SettingValue {
        self.default_value.as_ref()
    }

    pub fn value(&self) -> &dyn SettingValue {
        self.current_value
            .as_ref()
            .map(|v| v.as_ref())
            .unwrap_or(self.default_value.as_ref())
    }

    pub fn set(&mut self, value: Box<dyn SettingValue>) {
        self.current_value = Some(value);
    }

    pub fn set_value<T: SettingValue + 'static>(&mut self, value: T) {
        self.current_value = Some(Box::new(value));
    }

    pub fn reset(&mut self) {
        self.current_value = None;
    }

    pub fn is_default(&self) -> bool {
        self.current_value.is_none()
    }

    pub fn get_bool(&self) -> bool {
        self.value()
            .as_any()
            .downcast_ref::<bool>()
            .copied()
            .unwrap_or(false)
    }

    pub fn get_i32(&self) -> i32 {
        self.value()
            .as_any()
            .downcast_ref::<i32>()
            .copied()
            .unwrap_or(0)
    }

    pub fn get_i64(&self) -> i64 {
        self.value()
            .as_any()
            .downcast_ref::<i64>()
            .copied()
            .unwrap_or(0)
    }

    pub fn get_f64(&self) -> f64 {
        self.value()
            .as_any()
            .downcast_ref::<f64>()
            .copied()
            .unwrap_or(0.0)
    }

    pub fn get_string(&self) -> String {
        self.value()
            .as_any()
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_else(|| self.value().to_value_string())
    }
}

impl std::fmt::Debug for Setting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Setting")
            .field("id", &self.id())
            .field("synonyms", &self.synonyms)
            .finish()
    }
}
