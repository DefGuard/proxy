# build release binary
build:
    cargo build --release
# move tag to current commit
move-tag TAG:
    # remove local tag
    git tag --delete {{TAG}}
    # remove tag from remote
    git push --delete origin {{TAG}}
    # make new tag
    git tag {{TAG}}
    # push commits to remote
    git push
    # push new tag to remote
    git push origin {{TAG}}
