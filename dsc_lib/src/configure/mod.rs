// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use jsonschema::JSONSchema;

use crate::dscerror::DscError;
use crate::dscresources::dscresource::Invoke;
use crate::discovery::Discovery;
use self::config_doc::Configuration;
use self::depends_on::get_resource_invocation_order;
use self::config_result::{ConfigurationGetResult, ConfigurationSetResult, ConfigurationTestResult, ConfigurationExportResult, ResourceMessage, MessageLevel};
use std::collections::HashMap;

pub mod config_doc;
pub mod config_result;
pub mod depends_on;

pub struct Configurator {
    config: String,
    discovery: Discovery,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorAction {
    Continue,
    Stop,
}

impl Configurator {
    /// Create a new `Configurator` instance.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use in JSON.
    ///
    /// # Errors
    ///
    /// This function will return an error if the configuration is invalid or the underlying discovery fails.
    pub fn new(config: &str) -> Result<Configurator, DscError> {
        let mut discovery = Discovery::new()?;
        discovery.initialize()?;
        Ok(Configurator {
            config: config.to_owned(),
            discovery,
        })
    }

    /// Invoke the get operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `error_action` - The error action to use.
    /// * `progress_callback` - A callback to call when progress is made.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_get(&self, _error_action: ErrorAction, _progress_callback: impl Fn() + 'static) -> Result<ConfigurationGetResult, DscError> {
        let (config, messages, had_errors) = self.validate_config()?;
        let mut result = ConfigurationGetResult::new();
        result.messages = messages;
        result.had_errors = had_errors;
        if had_errors {
            return Ok(result);
        }
        for resource in get_resource_invocation_order(&config)? {
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type).next() else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            //TODO: add to debug stream:println!("{}", &resource.resource_type);
            let filter = serde_json::to_string(&resource.properties)?;
            let get_result = dsc_resource.get(&filter)?;
            let resource_result = config_result::ResourceGetResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: get_result,
            };
            result.results.push(resource_result);
        }

        Ok(result)
    }

    /// Invoke the set operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `error_action` - The error action to use.
    /// * `progress_callback` - A callback to call when progress is made.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_set(&self, _error_action: ErrorAction, _progress_callback: impl Fn() + 'static) -> Result<ConfigurationSetResult, DscError> {
        let (config, messages, had_errors) = self.validate_config()?;
        let mut result = ConfigurationSetResult::new();
        result.messages = messages;
        result.had_errors = had_errors;
        if had_errors {
            return Ok(result);
        }
        for resource in get_resource_invocation_order(&config)? {
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type).next() else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            //TODO: add to debug stream:println!("{}", &resource.resource_type);
            let desired = serde_json::to_string(&resource.properties)?;
            let set_result = dsc_resource.set(&desired)?;
            let resource_result = config_result::ResourceSetResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: set_result,
            };
            result.results.push(resource_result);
        }

        Ok(result)
    }

    /// Invoke the test operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `error_action` - The error action to use.
    /// * `progress_callback` - A callback to call when progress is made.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_test(&self, _error_action: ErrorAction, _progress_callback: impl Fn() + 'static) -> Result<ConfigurationTestResult, DscError> {
        let (config, messages, had_errors) = self.validate_config()?;
        let mut result = ConfigurationTestResult::new();
        result.messages = messages;
        result.had_errors = had_errors;
        if had_errors {
            return Ok(result);
        }
        for resource in get_resource_invocation_order(&config)? {
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type).next() else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            //TODO: add to debug stream:println!("{}", &resource.resource_type);
            let expected = serde_json::to_string(&resource.properties)?;
            let test_result = dsc_resource.test(&expected)?;
            let resource_result = config_result::ResourceTestResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: test_result,
            };
            result.results.push(resource_result);
        }

        Ok(result)
    }

    pub fn invoke_export(&self, _error_action: ErrorAction, _progress_callback: impl Fn() + 'static) -> Result<ConfigurationExportResult, DscError> {
        let (config, messages, had_errors) = self.validate_config()?;
        let mut result = ConfigurationExportResult::new();
        result.messages = messages;
        result.had_errors = had_errors;
        if had_errors {
            return Ok(result);
        };
        let mut conf = config_doc::Configuration::new();

        for resource in &config.resources {
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type).next() else {
                return Err(DscError::ResourceNotFound(resource.resource_type.clone()));
            };
            let export_result = dsc_resource.export()?;

            for (i, instance) in export_result.actual_state.iter().enumerate()
            {
                let mut r = config_doc::Resource::new();
                r.resource_type = dsc_resource.type_name.clone();
                r.name = format!("{}-{i}", r.resource_type);
                let props: HashMap<String, serde_json::Value> = serde_json::from_value(instance.clone()).unwrap();
                r.properties = Some(props);

                conf.resources.push(r);
            }

        }

        result.result = Some(conf);

        Ok(result)
    }

    fn validate_config(&self) -> Result<(Configuration, Vec<ResourceMessage>, bool), DscError> {
        let config: Configuration = serde_json::from_str(self.config.as_str())?;
        let mut messages: Vec<ResourceMessage> = Vec::new();
        let mut has_errors = false;
        for resource in &config.resources {
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type).next() else {
                return Err(DscError::ResourceNotFound(resource.resource_type.clone()));
            };

            //TODO: add to debug stream: println!("validate_config - resource_type - {}", resource.resource_type);
            //TODO: remove this after schema validation for classic PS resources is implemented
            if resource.resource_type == "DSC/PowerShellGroup" {continue;}

            let input = serde_json::to_string(&resource.properties)?;
            let schema = match dsc_resource.schema() {
                Ok(schema) => schema,
                Err(DscError::SchemaNotAvailable(_) ) => {
                    messages.push(ResourceMessage {
                        name: resource.name.clone(),
                        resource_type: resource.resource_type.clone(),
                        message: "Schema not available".to_string(),
                        level: MessageLevel::Warning,
                    });
                    continue;
                },
                Err(e) => {
                    return Err(e);
                },
            };
            let schema = serde_json::from_str(&schema)?;
            let compiled_schema = match JSONSchema::compile(&schema) {
                Ok(schema) => schema,
                Err(e) => {
                    messages.push(ResourceMessage {
                        name: resource.name.clone(),
                        resource_type: resource.resource_type.clone(),
                        message: format!("Failed to compile schema: {e}"),
                        level: MessageLevel::Error,
                    });
                    has_errors = true;
                    continue;
                },
            };
            let input = serde_json::from_str(&input)?;
            if let Err(err) = compiled_schema.validate(&input) {
                let mut error = format!("Resource '{}' failed validation: ", resource.name);
                for e in err {
                    error.push_str(&format!("\n{e} "));
                }
                messages.push(ResourceMessage {
                    name: resource.name.clone(),
                    resource_type: resource.resource_type.clone(),
                    message: error,
                    level: MessageLevel::Error,
                });
                has_errors = true;
                continue;
            };
        }

        Ok((config, messages, has_errors))
    }
}
