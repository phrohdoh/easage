set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cargo rustc --features $FEATURES --target $TARGET --bin $CRATE_NAME --release -- -C lto

    cp target/$TARGET/release/$CRATE_NAME $stage/

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
