# --- PROJECT CARTESIAN BASH CONFIG ---

# Source global definitions
if [ -f /etc/bash.bashrc ]; then
    . /etc/bash.bashrc
fi

# --- ALIASES ---
alias ls='ls --color=auto --group-directories-first'
alias ll='ls -lah'
alias l='ls -CF'
alias cr='cargo run'
alias cb='cargo build'
alias ct='cargo test'
alias cn='cargo new'
alias cshare='cd ~/share'

# Safety
alias cp='cp -i'
alias mv='mv -i'
alias rm='rm -i'

# --- PROMPT ---
# Export PS1 for a clean minimalist prompt
export PS1='\[\e[1;36m\][cartesian@arch \W]\$ \[\e[0m\] '

# Ensure Cargo binaries are in the PATH
export PATH="$HOME/.cargo/bin:$PATH"