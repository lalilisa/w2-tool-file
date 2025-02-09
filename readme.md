# Project Name

<span style="font-size: 1.5em;"> 
    Tool handle file with Rust using clap CLI. 
</span>

# Install

```
    git clone 
    
    cd clap-tool-file
    
    cargo install --path . 
```


# Usage

<span style="font-size: 1.5em;"> 
File Utility Tool</span>

````
Usage: clap-tool-file <COMMAND>

Commands:
  tree            List files and directories with sizes
  search          Search files containing specific pattern
  replace         Replace string in files
  count           Count lines matching pattern
  rename          Rename file
  list            List all files in current dic. Alias : ls
  remove          Remove file or dic. Alias : rm
  read-table-csv  read table csv
  copy            Copy file or dic. Alias : cp
  decompress-zip  Decompress file .zip
  decompress-gz   Decompress file .gz
  compress-zip    compress files zip
  compress-gz     compress files .gz
  csv-to-json     Convert csv to json
  json-to-csv     Convert json to csv
  read-file       Convert json to csv
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
````

# Examples

1. **List files and directories:**

   ```bash
   clap-tool-file tree ${path_dir}

2. **Read file:**

   ```bash
   clap-tool-file read ${path_file}
   
3. **Search for a pattern in files:**

   ```bash
   clap-tool-file search ${pattern_file} ${pattern_search}

4. **Replace string in files:**

   ```bash
   clap-tool-file replace ${path_file} --old_string ${old_string} --new_string ${new_string}

5. **Count lines matching pattern:**

   ```bash
   clap-tool-file count --path ${path_file} --pattern ${pattern} --r

6. **Rename file:**

   ```bash
   clap-tool-file rename ${old_file_name} ${new_file_name}
   
7. **List all files in current dic. Alias : ls:**

   ```bash
   clap-tool-file ls

8. **Remove file or dic. Alias : rm:**

   ```bash
   clap-tool-file rm --path ${path} --option ${option}
   ```
   ```
    Note: value option
    -r: remove only dic
    -f: remove only file
    -rf: remove file and dic
   ```

9. **read table csv**

   ```bash
   clap-tool-file read-csv-table ${path_file} --start ${start_row} --limit ${limit_row_show}

10. **Copy file or dic. Alias : cp:**
 
    **Copy file:**
    ```bash
    clap-tool-file cp ${path} ${copy_des}
    ```
    
    **Copy dir:**
    ```bash
    clap-tool-file cp ${path} ${copy_des} -r

11. **Decompress file .zip:**

    ```bash
    clap-tool-file decompress-zip ${path_zip_file}

12. **Decompress file .gz**

    ```bash
    clap-tool-file decompress-gz ${path_gz_file}

13. **Compress files zip:**

    ```bash
    clap-tool-file compress-zip --files ${file_1} ${file_2} ${dir_1}

14. **Compress files .gz:**

    ```bash
    clap-tool-file compress-gz --files ${file_1} ${file_2} ${dir_1}

15. **Convert csv to json:**

    ```bash
    clap-tool-file csv-to-json ${path_csv} ${output_json_path}


16. **Convert json to csv:**

    ```bash
    clap-tool-file json-to-csv ${path_json} ${output_csv_path}
    ```
