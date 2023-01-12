

rm -r ../gridlock-web/pkg
rm ../gridlock-web/index.html
rm ../gridlock-web/gridlock_worker.js

cp -r pkg ../gridlock-web
cp index.html ../gridlock-web
cp gridlock_worker.js ../gridlock-web
