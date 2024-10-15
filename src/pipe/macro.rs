#[macro_export]
macro_rules! make_pipe_error {
    ($expected:ident, $got:ident) => {
        PipeError::InvalidData {
            expected: PipeDataType::$expected,
            got: PipeDataType::$got,
        }
    };
}

#[macro_export]
macro_rules! match_pipe_data_to {
    ($to:expr,$type:ident, $expected:ident ) => {
        match $to {
            PipeableData::$type(m) => Ok(m),
            _ => Err(PipeError::InvalidData {
                expected: PipeDataType::$expected,
                got: $to.get_type(),
            }),
        }
    };
}
