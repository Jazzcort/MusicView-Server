cargo build -r --target=x86_64-unknown-linux-gnu
if [$? -ne 0]; then
	echo "Cargo build failed. Exiting script."
	exit 1
fi

ec2_instance="ec2-user@ec2-54-196-149-224.compute-1.amazonaws.com"

scp -i ~/jazzcort.com/jazzcort.pem ./target/x86_64-unknown-linux-gnu/release/actix-mongo ${ec2_instance}:./actix-mongo-server/.

