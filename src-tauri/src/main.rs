#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Substitua 'c_launcher' pelo nome do seu pacote no Cargo.toml se for diferente
    c_launcher::run();
}
