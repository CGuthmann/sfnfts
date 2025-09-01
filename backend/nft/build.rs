#[cfg(not(feature = "rapid_witnesscalc"))]
fn main(){}

#[cfg(feature = "rapid_witnesscalc")]
fn main() {
        println!("cargo:rustc-link-search=native=/usr/lib/gcc/x86_64-linux-gnu/10/"); //path to libstdc++.a
        println!("cargo:rustc-link-search=native=/usr/lib/sfnft-libs");
        

        println!("cargo:rustc-link-lib=static=stdc++");
        println!("cargo:rustc-link-lib=static=fr");
        println!("cargo:rustc-link-lib=static=gmp");

        println!("cargo:rustc-link-lib=static=nft_mint");
        println!("cargo:rustc-link-lib=static=nft_burn");
        println!("cargo:rustc-link-lib=static=nft_transfer");

}
