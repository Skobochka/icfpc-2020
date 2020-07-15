node {
  ansiColor('xterm') {
    stage ('Checkout') {
      checkout scm

      sh 'wget -O Dockerfile https://raw.githubusercontent.com/icfpcontest2020/dockerfiles/master/dockerfiles/rust/Dockerfile'
    }

    timeout(time: 10, unit: 'MINUTES') {
      stage('Build') {
        docker.build("icfpc2020-rust-org-image:${env.BUILD_TAG}", "--network=none .")
      }
    }

    // DISABLED until proper server found
    // stage('Smoke Test') {
    //   sh "docker run -t --rm --network=none -e RUST_BACKTRACE=1 icfpc2020-rust-org-image:${env.BUILD_TAG} http://server:12345 2933935384595749692"
    // }

    stage('Test') {
      sh "docker run -t --rm --network=none -e RUST_BACKTRACE=1 --entrypoint ./test.sh icfpc2020-rust-org-image:${env.BUILD_TAG}"
    }

    // This is how Orgs will run the solution.
    // stage('Run') {
    //   echo 'Running solver...'
    //   sh "docker run -t --rm icfpc2020-rust-org-image:${env.BUILD_TAG}"
    // }

    stage('Cleanup') {
      sh "docker rmi icfpc2020-rust-org-image:${env.BUILD_TAG}"
    }
  }
}
