# The frontespizio title page is typeset in a separate document
# (main_thesis-frn.tex) that the main run compiles via \write18.
$pdflatex = 'pdflatex -shell-escape %O %S';
$pdf_mode = 1;
