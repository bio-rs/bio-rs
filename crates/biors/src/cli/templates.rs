use super::TemplateCommand;
use crate::errors::CliError;
use crate::output::print_success;
use biors_core::templates::{find_task_template, task_templates};

pub(crate) fn run_template_command(command: TemplateCommand) -> Result<(), CliError> {
    match command {
        TemplateCommand::List => print_success(None, task_templates()),
        TemplateCommand::Show { id } => match find_task_template(&id) {
            Some(template) => print_success(None, template),
            None => Err(CliError::Validation {
                code: "template.not_found",
                message: format!("task template '{id}' was not found"),
                location: Some(id),
            }),
        },
    }
}
