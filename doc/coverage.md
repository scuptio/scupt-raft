# Test coverage in docker container

## run build images

In the project folder,

1. Build test image:
    ```shell
    docker build . -f test.dockerfile -t scupt-raft-test:latest
    ```

2. Run container web server:
    ```shell
    docker run --name scupt-raft-test --publish 8000:8000 --detach scupt-raft-test
    ```

   See http://127.0.0.1:8000/ for coverage report
