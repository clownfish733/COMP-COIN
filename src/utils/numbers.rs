pub fn format_number(n: usize) -> String{
    if n < 1_000_000{
        format!("{},000", n/1_000)
    }else{
        format!("{},{:03},000", n/1_000_000, (n%1_000_000)/1_000)
    }
}