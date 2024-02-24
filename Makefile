
build: clean
	npm i
	npx tailwindcss -i ./styles.css -o ./out/styles.css -m
	cargo run

format: 
	cargo fmt
	npx prettier -w **/*.{css,html,js,json,md}

clean:
	rm -rf ./out
