use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main() {
    println!("🎮 Welcome to the Guessing Game!");

    let secret_number = rand::thread_rng().gen_range(1..=10);
    let mut attempts = 0;

    loop {
        println!("\nEnter your guess (1–10):");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("❌ Please enter a valid number!");
                continue;
            }
        };

        // Range validation
        if guess < 1 || guess > 10 {
            println!("⚠️ Number must be between 1 and 10!");
            continue;
        }

        attempts += 1;

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("📉 Too small!"),
            Ordering::Greater => println!("📈 Too big!"),
            Ordering::Equal => {
                println!("🎉 You win!");
                println!("✅ Attempts: {}", attempts);
                break;
            }
        }
    }
}