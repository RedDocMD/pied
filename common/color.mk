define colorecho
      @tput setaf 6 2> /dev/null || true
      @echo -e $1
      @tput sgr0 2> /dev/null || true
endef
