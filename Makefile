
build: clean
	npm i
	npx tailwindcss -i ./styles.css -o ./out/styles.css -m
	cargo run

format: 
	cargo fmt
	npx prettier -w **/*.html

clean:
	rm -rf ./out
