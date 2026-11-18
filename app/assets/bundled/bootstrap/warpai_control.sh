# The bootstrap owns this session only. Never source code from the installation.
function __warpai_enable_control {
    local executable native
    case $- in *i*) ;; *) return 1 ;; esac
    [ -t 0 ] || return 1
    [ -z "${__warpai_ctl_executable+x}" ] || return 1
    case ${HOME:-} in /*) ;; *) return 1 ;; esac
    case $HOME in *[[:cntrl:]]*) return 1 ;; esac
    if [ "${WARP_IS_SSH:-}" = 1 ]; then
        executable="$HOME/.config/warpai/bin/warpai-companion"
    elif [ -n "${WSL_DISTRO_NAME:-}" ] && [ -x /usr/bin/wslpath ]; then
        native=$(/usr/bin/wslpath -w / 2>/dev/null) || return 1
        case $native in
            '\\wsl.localhost\'"$WSL_DISTRO_NAME"|'\\wsl.localhost\'"$WSL_DISTRO_NAME"'\'|'\\wsl$\'"$WSL_DISTRO_NAME"|'\\wsl$\'"$WSL_DISTRO_NAME"'\') ;;
            *) return 1 ;;
        esac
        executable="$HOME/.config/warpai/wsl/bin/warpai-wsl-companion"
    else
        return 1
    fi
    [ -f "$executable" ] && [ -x "$executable" ] && [ ! -L "$executable" ] || return 1
    if command -v warpai >/dev/null 2>&1; then
        printf 'Warpai control is also available at %s ctl\n' "$executable" >&2
        return 1
    fi
    __warpai_ctl_executable="$executable"
    function warpai {
        local executable="$__warpai_ctl_executable"
        local directory="${__warpai_ctl_executable%/*}"
        local manifest="$directory/companion-manifest.json"
        local account="${directory%/.config/warpai/*}"
        local owner entry manifest_size expected actual
        owner=$(command -p id -u) || return 1
        entry=$directory
        while :; do
            if [ -L "$entry" ] || [ "$(command -p find "$entry" -prune -type d -user "$owner" ! -perm -020 ! -perm -002 -print 2>/dev/null)" != "$entry" ]; then
                printf '%s\n' 'Warpai Companion installation directory is unverified.' >&2
                return 1
            fi
            [ "$entry" = "$account" ] && break
            entry=${entry%/*}
        done
        for entry in "$executable" "$manifest"; do
            if [ -L "$entry" ] || [ "$(command -p find "$entry" -prune -type f -user "$owner" ! -perm -020 ! -perm -002 -print 2>/dev/null)" != "$entry" ]; then
                printf '%s\n' 'Warpai Companion installation file is unverified.' >&2
                return 1
            fi
        done
        manifest_size=$(command -p wc -c < "$manifest") || return 1
        [ "$manifest_size" -le 65536 ] || return 1
        expected=$(command -p sed -nE 's/.*"sha256"[[:space:]]*:[[:space:]]*"([0-9a-f]{64})".*/\1/p' "$manifest") || return 1
        if command -pv sha256sum >/dev/null 2>&1; then
            actual=$(command -p sha256sum < "$executable") || return 1
        else
            actual=$(command -p shasum -a 256 < "$executable") || return 1
        fi
        if [ "${actual%% *}" != "$expected" ]; then
            printf '%s\n' 'Warpai Companion checksum does not match its installed manifest.' >&2
            return 1
        fi
        command "$executable" ctl "$@"
    }
}
if __warpai_enable_control; then readonly __warpai_ctl_executable; fi
unset -f __warpai_enable_control
