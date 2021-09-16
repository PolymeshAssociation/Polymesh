./stop.sh
rm -rf /tmp/pmesh-*
npm run build
./run.sh
./test.sh
