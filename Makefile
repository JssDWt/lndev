
build: clean
	npm i
	npx tailwindcss -i ./styles.css -o ./out/styles.css -m
	cargo run

clean:
	rm -rf ./out
	rm -rf ./dist