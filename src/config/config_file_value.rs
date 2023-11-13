pub trait ConfigFileValue {
    type ConfigValue;

    fn to_config_value(self) -> Self::ConfigValue;
}

#[macro_export]
macro_rules! basic_config_file_value {
    ($name:ident: $config_value_type:ty) => {
        #[derive(Clone, Debug, Serialize)]
        pub struct $name {
            value: $config_value_type,
        }

        impl ConfigFileValue for $name {
            type ConfigValue = $config_value_type;

            fn to_config_value(self) -> Self::ConfigValue {
                self.value
            }
        }

        impl From<$config_value_type> for $name {
            fn from(value: $config_value_type) -> Self {
                Self { value: value }
            }
        }
    };
}

#[macro_export]
macro_rules! config_struct_pair {
    ($name:ident, $name_config_file:ident, $($field:ident: $type_config_file:ty,)*) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct $name {
            $(
                pub $field: <$type_config_file as crate::config::config_file_value::ConfigFileValue>::ConfigValue,
            )*
        }

        #[derive(Clone, Debug, Deserialize, Serialize)]
        pub struct $name_config_file {
            $(
                $field: $type_config_file,
            )*
        }

        impl ConfigFileValue for $name_config_file {
            type ConfigValue = $name;

            fn to_config_value(self) -> Self::ConfigValue {
                Self::ConfigValue {
                    $(
                        $field: self.$field.to_config_value(),
                    )*
                }
            }
        }
    };
}
