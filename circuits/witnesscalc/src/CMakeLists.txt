include_directories(
    ../src
    ../build
    ../depends/json/single_include)

link_libraries(${GMP_LIB})

add_definitions(${GMP_DEFINIONS})

if(USE_ASM)
    if(ARCH MATCHES "arm64")
            add_definitions(-DUSE_ASM -DARCH_ARM64)
    elseif(ARCH MATCHES "x86_64")
            add_definitions(-DUSE_ASM -DARCH_X86_64)
    endif()
endif()

if(USE_ASM AND ARCH MATCHES "x86_64")

    if (CMAKE_HOST_SYSTEM_NAME MATCHES "Darwin")
        set(NASM_FLAGS "-fmacho64 --prefix _")
    else()
        set(NASM_FLAGS -felf64)
    endif()

    add_custom_command(OUTPUT ${CMAKE_SOURCE_DIR}/build/fq_asm.o
        COMMAND nasm ${NASM_FLAGS} fq.asm -o fq_asm.o
        DEPENDS ${CMAKE_SOURCE_DIR}/build/fq.asm
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/build)

    add_custom_command(OUTPUT ${CMAKE_SOURCE_DIR}/build/fr_asm.o
        COMMAND nasm ${NASM_FLAGS} fr.asm -o fr_asm.o
        DEPENDS ${CMAKE_SOURCE_DIR}/build/fr.asm
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/build)
endif()

set(FR_SOURCES
    ../build/fr.hpp
    ../build/fr.cpp
    )

if(USE_ASM)
    if(ARCH MATCHES "arm64")
        set(FR_SOURCES ${FR_SOURCES} ../build/fr_raw_arm64.s ../build/fr_raw_generic.cpp ../build/fr_generic.cpp)
    elseif(ARCH MATCHES "x86_64")
        set(FR_SOURCES ${FR_SOURCES} ../build/fr_asm.o)
    endif()
else()
    set(FR_SOURCES ${FR_SOURCES} ../build/fr_generic.cpp ../build/fr_raw_generic.cpp)
endif()

add_library(fr STATIC ${FR_SOURCES})
set_target_properties(fr PROPERTIES POSITION_INDEPENDENT_CODE ON)

link_libraries(fr)


#add_executable(tests tests.cpp)
#add_executable(test_platform test_platform.cpp)


set(LIB_SOURCES
    calcwit.cpp
    witnesscalc.h
    witnesscalc.cpp
    )

# poseidon_bench
set(POSEIDON_BENCH_SOURCES ${LIB_SOURCES}
    poseidon_bench.cpp
    witnesscalc_poseidon_bench.h
    witnesscalc_poseidon_bench.cpp
    )
add_library(witnesscalc_poseidon_bench SHARED ${POSEIDON_BENCH_SOURCES})
add_library(witnesscalc_poseidon_benchStatic STATIC ${POSEIDON_BENCH_SOURCES})
set_target_properties(witnesscalc_poseidon_benchStatic PROPERTIES OUTPUT_NAME poseidon_bench)
add_executable(poseidon_bench main.cpp)
target_link_libraries(poseidon_bench witnesscalc_poseidon_benchStatic)
target_compile_definitions(witnesscalc_poseidon_bench PUBLIC CIRCUIT_NAME=poseidon_bench)
target_compile_definitions(witnesscalc_poseidon_benchStatic PUBLIC CIRCUIT_NAME=poseidon_bench)
target_compile_definitions(poseidon_bench PUBLIC CIRCUIT_NAME=poseidon_bench)

# nft minting
set(NFT_MINT_SOURCES ${LIB_SOURCES}
    nft_mint.cpp
    witnesscalc_nft_mint.h
    witnesscalc_nft_mint.cpp
    )
add_library(witnesscalc_nft_mint SHARED ${NFT_MINT_SOURCES})
add_library(witnesscalc_nft_mintStatic STATIC ${NFT_MINT_SOURCES})
set_target_properties(witnesscalc_nft_mintStatic PROPERTIES OUTPUT_NAME nft_mint)
add_executable(nft_mint main.cpp)
target_link_libraries(nft_mint witnesscalc_nft_mintStatic)
target_compile_definitions(witnesscalc_nft_mint PUBLIC CIRCUIT_NAME=nft_mint)
target_compile_definitions(witnesscalc_nft_mintStatic PUBLIC CIRCUIT_NAME=nft_mint)
target_compile_definitions(nft_mint PUBLIC CIRCUIT_NAME=nft_mint)

# nft burning
set(NFT_BURN_SOURCES ${LIB_SOURCES}
    nft_burn.cpp
    witnesscalc_nft_burn.h
    witnesscalc_nft_burn.cpp
    )
add_library(witnesscalc_nft_burn SHARED ${NFT_BURN_SOURCES})
add_library(witnesscalc_nft_burnStatic STATIC ${NFT_BURN_SOURCES})
set_target_properties(witnesscalc_nft_burnStatic PROPERTIES OUTPUT_NAME nft_burn)
add_executable(nft_burn main.cpp)
target_link_libraries(nft_burn witnesscalc_nft_burnStatic)
target_compile_definitions(witnesscalc_nft_burn PUBLIC CIRCUIT_NAME=nft_burn)
target_compile_definitions(witnesscalc_nft_burnStatic PUBLIC CIRCUIT_NAME=nft_burn)
target_compile_definitions(nft_burn PUBLIC CIRCUIT_NAME=nft_burn)

# nft transfering
set(NFT_TRANSFER_SOURCES ${LIB_SOURCES}
    nft_transfer.cpp
    witnesscalc_nft_transfer.h
    witnesscalc_nft_transfer.cpp
    )
add_library(witnesscalc_nft_transfer SHARED ${NFT_TRANSFER_SOURCES})
add_library(witnesscalc_nft_transferStatic STATIC ${NFT_TRANSFER_SOURCES})
set_target_properties(witnesscalc_nft_transferStatic PROPERTIES OUTPUT_NAME nft_transfer)
add_executable(nft_transfer main.cpp)
target_link_libraries(nft_transfer witnesscalc_nft_transferStatic)
target_compile_definitions(witnesscalc_nft_transfer PUBLIC CIRCUIT_NAME=nft_transfer)
target_compile_definitions(witnesscalc_nft_transferStatic PUBLIC CIRCUIT_NAME=nft_transfer)
target_compile_definitions(nft_transfer PUBLIC CIRCUIT_NAME=nft_transfer)



# nft transfer_childing
set(NFT_TRANSFER_CHILD_SOURCES ${LIB_SOURCES}
    nft_transfer_child.cpp
    witnesscalc_nft_transfer_child.h
    witnesscalc_nft_transfer_child.cpp
    )
add_library(witnesscalc_nft_transfer_child SHARED ${NFT_TRANSFER_CHILD_SOURCES})
add_library(witnesscalc_nft_transfer_childStatic STATIC ${NFT_TRANSFER_CHILD_SOURCES})
set_target_properties(witnesscalc_nft_transfer_childStatic PROPERTIES OUTPUT_NAME nft_transfer_child)
add_executable(nft_transfer_child main.cpp)
target_link_libraries(nft_transfer_child witnesscalc_nft_transfer_childStatic)
target_compile_definitions(witnesscalc_nft_transfer_child PUBLIC CIRCUIT_NAME=nft_transfer_child)
target_compile_definitions(witnesscalc_nft_transfer_childStatic PUBLIC CIRCUIT_NAME=nft_transfer_child)
target_compile_definitions(nft_transfer_child PUBLIC CIRCUIT_NAME=nft_transfer_child)




# nft child_parenting
set(NFT_CHILD_PARENT_SOURCES ${LIB_SOURCES}
    nft_child_parent.cpp
    witnesscalc_nft_child_parent.h
    witnesscalc_nft_child_parent.cpp
    )
add_library(witnesscalc_nft_child_parent SHARED ${NFT_CHILD_PARENT_SOURCES})
add_library(witnesscalc_nft_child_parentStatic STATIC ${NFT_CHILD_PARENT_SOURCES})
set_target_properties(witnesscalc_nft_child_parentStatic PROPERTIES OUTPUT_NAME nft_child_parent)
add_executable(nft_child_parent main.cpp)
target_link_libraries(nft_child_parent witnesscalc_nft_child_parentStatic)
target_compile_definitions(witnesscalc_nft_child_parent PUBLIC CIRCUIT_NAME=nft_child_parent)
target_compile_definitions(witnesscalc_nft_child_parentStatic PUBLIC CIRCUIT_NAME=nft_child_parent)
target_compile_definitions(nft_child_parent PUBLIC CIRCUIT_NAME=nft_child_parent)
