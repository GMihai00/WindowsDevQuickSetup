use super::common::{
    expand_string, expand_string_deserializer, set_install_value, InstallActionType,
};

use serde_derive::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use std::error::Error;

use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
use winreg::RegKey;

#[derive(Deserialize, Serialize)]
struct GetRegistryValueCommand {
    #[serde(deserialize_with = "expand_string_deserializer")]
    reg_path: String,

    #[serde(deserialize_with = "expand_string_deserializer")]
    key_name: String,

    #[serde(deserialize_with = "expand_string_deserializer")]
    install_key: String,
}

impl GetRegistryValueCommand {
    pub fn execute(&self, _action: &InstallActionType) -> Result<bool, Box<dyn Error>> {
        let hklm = RegKey::predef(HKEY_CURRENT_USER);

        let subkey = hklm.open_subkey_with_flags(&self.reg_path.as_str(), KEY_READ)?;

        match subkey.get_value::<String, _>(&self.key_name.as_str()) {
            Ok(string_value) => {
                set_install_value(
                    &self.install_key.as_str(),
                    expand_string(string_value.as_str()).as_str(),
                );
            }
            Err(_) => match subkey.get_value::<u32, _>(&self.key_name.as_str()) {
                Ok(dword_value) => set_install_value(&self.install_key.as_str(), dword_value),
                Err(err) => return Err(err.into()),
            },
        }

        return Ok(true);
    }
}

pub fn get_registry_value(
    json_data: &Value,
    action: &InstallActionType,
) -> Result<bool, Box<dyn Error>> {
    let cmd: GetRegistryValueCommand = from_value(json_data.clone())?;

    return cmd.execute(action);
}
