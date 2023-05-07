pub trait ParameterConstraint<T> {
    fn description(&self) -> String;
    fn is_valid(&self, value: &T) -> Result<(), String>;
}

struct CallbackConstraint<F> {
    description: String,
    callback: F,
}

impl<T, F> ParameterConstraint<T> for CallbackConstraint<F>
where
    F: Fn(&T) -> Result<(), String>,
{
    fn description(&self) -> String {
        self.description.clone()
    }
    fn is_valid(&self, value: &T) -> Result<(), String> {
        (self.callback)(value)
    }
}

pub struct ExactLengthConstraint {
    len: usize,
}

impl ParameterConstraint<Vec<u8>> for ExactLengthConstraint {
    fn description(&self) -> String {
        format!("Array length must be equal to {}", self.len)
    }
    fn is_valid(&self, value: &Vec<u8>) -> Result<(), String> {
        if value.len() == self.len {
            Ok(())
        } else {
            Err(format!(
                "Rejected array because its length is {}, but a length of {} is required",
                value.len(),
                self.len
            ))
        }
    }
}

pub type Constraints<T> = Vec<Box<dyn ParameterConstraint<T>>>;
