use super::common::{InstallActionType, ActionFn};

use std::error::Error;

use serde_json::{from_value, json, Value};
use serde_derive::{Deserialize, Serialize};

use tokio::task;
use futures::future;

use super::super::rendering::render;

#[derive(Deserialize, Serialize)]
struct ParalelExecCommand {
    run: Value
}

impl ParalelExecCommand {
    pub async fn execute(&self, action: &InstallActionType) -> Result<bool, Box<dyn Error  + Send + Sync>> {
        if let Value::Array(obj) = &self.run {
        
            let mut tasks = vec![];
            for value in obj.iter() {
                if let Value::Object(object) = value {
                    if object.len() != 1 {
                        let json_string =
                            serde_json::to_string(&object).expect("Failed to convert JSON to string");
                        panic!(
                            "Failed to found matching instruction for json: {}",
                            json_string
                        );
                    }
    
                    let encapsulated_command = json!([object.clone()]);
                    let cpy_action = action.clone();
                    let task = task::spawn( async move {
                        return render(&encapsulated_command, &cpy_action).await;
                    });
                    tasks.push(task);
                }
            }
            
            let (item_resolved, idx, remaining_futures) = future::select_all(tasks).await;
            
            match item_resolved {
                Ok(value) => {
                    match value {
                        Ok(ret) => { if !ret{
                            println!("One of the paralel ran commands failed");
                            return Ok(false)
                        }}
                        Err(err) => { return Err(err); }
                    }
                }
                Err(err) => {
                    panic!("Task at index {} failed with error: {:?}", idx, err);
                }
            }
            
            let mut futures = remaining_futures;

            while !futures.is_empty() {
                let (item_resolved, idx, remaining_futures) = future::select_all(futures).await;
        
                match item_resolved {
                    Ok(value) => {
                        match value {
                            Ok(ret) => { if !ret{
                                println!("One of the paralel ran commands failed");
                                return Ok(false)
                            }}
                            Err(err) => { return Err(err); }
                        }
                    }
                    Err(err) => {
                        panic!("Task at index {} failed with error: {:?}", idx, err);
                    }
                }
        
                futures = remaining_futures;
            }
        }
        else {
            panic!("Invalid syntax, comands are supposed to be contained into an array of objects");
        }

        return Ok(true);
    }
}

pub struct ParalelExecCommandExecutor{
}

use async_trait::async_trait;

#[async_trait]
impl ActionFn for ParalelExecCommandExecutor{
    async fn execute_command(&self, json_data: &Value, action: &InstallActionType) -> Result<bool, Box<dyn Error  + Send + Sync>>
    {
        let cmd: ParalelExecCommand = from_value(json_data.clone())?;

        return cmd.execute(action).await;
    }
}
