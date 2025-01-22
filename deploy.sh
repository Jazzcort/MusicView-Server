cargo build -r --target=x86_64-unknown-linux-gnu
if [$? -ne 0]; then
	echo "Cargo build failed. Exiting script."
	exit 1
fi

ec2_instance="ec2-user@ec2-54-196-149-224.compute-1.amazonaws.com"
key_file="~/jazzcort.com/jazzcort.pem"

scp -i $key_file ./target/x86_64-unknown-linux-gnu/release/actix-mongo ${ec2_instance}:./actix-mongo-server/tmp && \
ssh -i $key_file $ec2_instance << 'ENDSSH'
sudo -i
rm -f "/home/ec2-user/actix-mongo-server/actix-mongo" && \
mv "/home/ec2-user/actix-mongo-server/tmp" "/home/ec2-user/actix-mongo-server/actix-mongo" && \
systemctl restart actix-mongo
ENDSSH


