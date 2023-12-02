macro_rules! common_string {
    ($t:ty, $err:ty) => {
        impl From<$t> for String {
            fn from(value: $t) -> String {
                value.0
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl PartialEq<str> for $t {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl<'a> PartialEq<&'a str> for $t {
            fn eq(&self, other: &&'a str) -> bool {
                &self.0 == other
            }
        }

        impl AsRef<str> for $t {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl std::ops::Deref for $t {
            type Target = str;

            fn deref(&self) -> &str {
                &*self.0
            }
        }

        impl std::str::FromStr for $t {
            type Err = $err;

            fn from_str(s: &str) -> Result<$t, $err> {
                String::from(s).try_into()
            }
        }
    };
}
