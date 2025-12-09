fn main() {
    cc::Build::new()
        .include("src/swisseph")
        .file("src/swisseph/sweph.c")
        .file("src/swisseph/swephlib.c")
        .file("src/swisseph/swedate.c")
        .file("src/swisseph/swehouse.c")
        .file("src/swisseph/swecl.c")
        .file("src/swisseph/swehel.c")
        .file("src/swisseph/swejpl.c")
        .file("src/swisseph/swemmoon.c")
        .file("src/swisseph/swemplan.c")
        .compile("swisseph");
}
