### pipe-gpt for en mass single-file code reviews
find ./src -name '*.rs' | while read -r file; do echo "File: $file" && cat "$file" | pipe-gpt --markdown -p "Can this code be improved for efficiency?"; done

find ./src -name '*.rs' | while read -r file; do echo "File: $file" && cat "$file" | pipe-gpt --markdown -p "Can this code be improved for readability?"; done
