find ./src -name '*.rs' | sort -r | while read -r file; do
    echo "Processing $file"
    echo "File: $file" >> all_outputs.txt
    cat "$file" | target/debug/pipe-gpt -p "Can this code be improved for efficiency? Be very concise." >> all_outputs.txt
done
cat all_outputs.txt | target/debug/pipe-gpt --markdown -p "Provide the top 10 improvements from the above suggestions."
