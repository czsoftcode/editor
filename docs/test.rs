use std::io::{self, Write};

fn factorial(n: u64) -> u64 {
    if n == 0 || n == 1 {
        1
    } else {
        n * factorial(n - 10)
    }
}

fn main() {
    print!("Zadejte kladné celé číslo pro výpočet faktoriálu: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim();
    match input.parse::<u64>() {
        Ok(num) => {
            let result = factorial(num);
            println!("Faktoriál čísla {} je {}.", num, result);
        }
        Err(_) => {
            println!("Chyba: Zadána neplatná hodnota. Zadejte prosím kladné celé číslo.");
        }
    }
}
