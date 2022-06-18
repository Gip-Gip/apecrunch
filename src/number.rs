
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
// Placeholder number struct, will be converted to a fractional number later...
pub struct Number
{
    value: i64
}

impl Number
{   
    pub fn from_str(s: &str) -> Result<Self, Box<dyn Error>>
    {
        let value = s.parse::<i64>()?;
        
        return Ok(Number{
            value: value
        });
    }

    pub fn to_string(&self) -> String
    {
        return self.value.to_string();
    }

    pub fn add(&self, other: &Number) -> Number
    {
        return Number{
            value: self.value + other.value
        };
    }

    pub fn subtract(&self, other: &Number) -> Number
    {
        return Number{
            value: self.value - other.value
        };
    }

    pub fn multiply(&self, other: &Number) -> Number
    {
        return Number{
            value: self.value * other.value
        };
    }

    pub fn divide(&self, other: &Number) -> Number
    {
        return Number{
            value: self.value / other.value
        };
    }
}