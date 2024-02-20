
build: clean
	npm i
	npx tailwindcss -i ./styles.css -o ./out/styles.css
	cargo run

clean:
	rm -rf ./out
	rm -rf ./dist