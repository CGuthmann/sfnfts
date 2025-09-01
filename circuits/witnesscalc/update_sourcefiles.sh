sed -i '6i namespace CIRCUIT_NAME{' src/nft_mint.cpp
sed -i '6i namespace CIRCUIT_NAME{' src/nft_transfer.cpp
sed -i '6i namespace CIRCUIT_NAME{' src/nft_transfer_child.cpp
sed -i '6i namespace CIRCUIT_NAME{' src/nft_child_parent.cpp
sed -i '6i namespace CIRCUIT_NAME{' src/nft_burn.cpp

echo '}' >> src/nft_mint.cpp
echo '}' >> src/nft_transfer.cpp
echo '}' >> src/nft_transfer_child.cpp
echo '}' >> src/nft_child_parent.cpp
echo '}' >> src/nft_burn.cpp