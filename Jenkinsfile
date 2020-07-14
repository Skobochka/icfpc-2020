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
  
    stage('Test') {
      echo 'Starting tests...'
      sh "docker run -t --rm --network=none -e RUST_BACKTRACE=1 icfpc2020-rust-org-image:${env.BUILD_TAG}"
    }
  }
}
