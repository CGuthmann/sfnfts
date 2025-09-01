cmake_minimum_required(VERSION 3.5)

include(cmake/platform.cmake)

set(USE_ASM ON CACHE BOOL "Use asm implementation for Fr and Fq")

project(witnesscalc LANGUAGES CXX ASM)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

message("USE_ASM=" ${USE_ASM})
message("CMAKE_CROSSCOMPILING=" ${CMAKE_CROSSCOMPILING})

message("GMP_PREFIX=" ${GMP_PREFIX})
message("GMP_INCLUDE_DIR=" ${GMP_INCLUDE_DIR})
message("GMP_LIB_DIR=" ${GMP_LIB_DIR})

if (NOT EXISTS ${GMP_INCLUDE_FILE_FULLPATH})
    message("WARNING: ${GMP_INCLUDE_FILE_FULLPATH} is not found and so system ${GMP_INCLUDE_FILE} is used.")
endif()

if (NOT EXISTS ${GMP_LIB_FILE_FULLPATH})
    message("WARNING: ${GMP_LIB_FILE_FULLPATH} is not found and so system ${GMP_LIB_FILE} is used.")
    set(GMP_LIB gmp)
endif()


include_directories(BEFORE ${GMP_INCLUDE_DIR})

add_subdirectory(src)


install(TARGETS
    poseidon_bench
    witnesscalc_poseidon_bench
    witnesscalc_poseidon_benchStatic
    nft_mint
    witnesscalc_nft_mint
    witnesscalc_nft_mintStatic
    nft_burn
    witnesscalc_nft_burn
    witnesscalc_nft_burnStatic
    nft_transfer
    witnesscalc_nft_transfer
    witnesscalc_nft_transferStatic
    nft_transfer_child
    witnesscalc_nft_transfer_child
    witnesscalc_nft_transfer_childStatic
    nft_child_parent
    witnesscalc_nft_child_parent
    witnesscalc_nft_child_parentStatic
    fr
    RUNTIME DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
    BUNDLE DESTINATION ${CMAKE_INSTALL_PREFIX}/app
    LIBRARY DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)

install(FILES "${GMP_LIB_DIR}/${GMP_LIB_FILE}"
    DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)

install(FILES
    src/nft_mint.dat
    src/nft_burn.dat
    src/nft_transfer.dat
    src/nft_transfer_child.dat
    src/nft_child_parent.dat
    src/poseidon_bench.dat
    DESTINATION ${CMAKE_INSTALL_PREFIX}/bin)

install(FILES
    src/witnesscalc.h
    src/witnesscalc_nft_mint.h
    src/witnesscalc_nft_burn.h
    src/witnesscalc_nft_transfer.h
    src/witnesscalc_nft_child_parent.h
    src/witnesscalc_nft_transfer_child.h
    src/witnesscalc_poseidon_bench.h
    DESTINATION ${CMAKE_INSTALL_PREFIX}/include)
