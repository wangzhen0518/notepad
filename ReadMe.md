# TODO

-   [ ] 自动换行
-   [ ] 鼠标操作
-   [ ] Support more filetypes
-   [ ] Make highlighting markers configurable. For instance, in some languages, a single # instead of two slashes indicate a single line comment.
-   [ ] Allow alternatives. For instance, in many languages, characters are not highlighted separately, but instead, strings can be delimited with single or double quotes.
-   [ ] Line numbers: Display the line number to the left of each line of the file.
-   [ ] Auto indent: When starting a new line, indent it to the same level as the previous line.+
-   [ ] Hard-wrap lines: Insert a newline in the text when the user is about to type past the end of the screen. Try not to insert the newline where it would split up a word.
-   [ ] Soft-wrap lines: When a line is longer than the screen width, use multiple lines on the screen to display it instead of horizontal scrolling.
-   [ ] Better handling of indices: We have been a bit indicisive about when to use saturating_add and similar functions and when to do pointer arithmetic. We also have not done a good job at safely and consistently accessing entries of a Vec.
-   [ ] Copy and paste: Give the user a way to select text, and then copy the selected text when they press Ctrl-C, and let them paste the copied text when they press Ctrl-V.
-   [ ] Multiple buffers: Allow having multiple files open at once, and have some way of switching between them.
-   [ ] Take a look at the Pull Requests for hecto and check out which bugs they are fixing and when they were introduced.
