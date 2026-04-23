use include_tt::inject;

macro_rules! _exp_shielding {
    [ -- #ctt ($($_path:tt)*) ] => {
        macro_rules! _check_eq {
            ( "./tests/invalid_token.tt" ) => {}; // OK
            ( $_unknown2: tt ) => { // ERR
                compile_error!("Escaping the `#` character does not work, check the `inject` macro.");
            }
        }

        _check_eq!($($_path)*);
    };
    [ $($_unknown:tt)* ] => {
        compile_error!("Escaping the `#` character does not work, check the `inject` macro.");
    }
}

#[test]
fn test_shielding() {
    // line_1: `--` checking that the `-` character only escapes `#`
    // line_2: `-#` verifies the actual character escaping operation.
    //              If everything is correct, `#ctt("./tests/invalid_token.tt")` will be included in the macro body as is.
    //
    inject! {
        _exp_shielding! {
            --
            -#ctt("./tests/invalid_token.tt")
        }
    }
}
