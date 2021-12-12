fn main() {
    let db = satisfactory_accounting::Database::instance();
    for item in db.items.values() {
        println!("{}", item.name);
        println!("  Produced By:");
        for recipe in item.produced_by.iter().map(|&r| &db[r]) {
            println!("  - {}", recipe.name);
        }
        println!("  Consumed By:");
        for recipe in item.consumed_by.iter().map(|&r| &db[r]) {
            println!("  - {}", recipe.name);
        }
    }
}
