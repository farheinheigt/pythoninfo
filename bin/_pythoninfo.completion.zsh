#compdef pythoninfo
_pythoninfo_completion() {
  _arguments \
    '(-h --help)'{-h,--help}'[Afficher l aide]' \
    '--completion[Generer la completion shell]:shell:(zsh)'
}

compdef _pythoninfo_completion pythoninfo
