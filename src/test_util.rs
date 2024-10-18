// pub type nom::IResult<I, O, E = error::Error<I>> = Result<(I, O), Err<E>>;
#[allow(unused)]
pub fn generic_command_parse<F, I, T, E>(parser: F, input: I, expect: T)
where
    F: Fn(I) -> Result<(I, T), E>,
    T: PartialEq + std::fmt::Debug,
    I: std::fmt::Debug,
    E: std::fmt::Debug,
{
    let result = parser(input);
    assert!(result.is_ok(), "The result:\r\n{:#?}", result);
    let (rest, res) = result.unwrap();
    assert_eq!(
        expect, res,
        "The expect:\r\n({:?}) should be same with the result:\r\n({:?})",
        expect, res
    );
}

#[allow(unused)]
pub fn generic_parse<F, I, T>(parser: F, input: I, expect: T)
where
    F: Fn(I) -> T,
    T: PartialEq + std::fmt::Debug,
    I: std::fmt::Debug,
{
    let result = parser(input);
    assert_eq!(
        expect, result,
        "The expect:\r\n({:?}) should be same with the result:\r\n({:?})",
        expect, result
    );
}
