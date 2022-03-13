import "other.rhai" as other;

fn main() {
    print("hello from script!");
    other::print_from_other();
    return 0;
}
