pipeline {
  agent none;
  stages {
    stage('Fetch Dockerfile') {
      agent any;
      steps {
        sh 'wget -O Dockerfile https://raw.githubusercontent.com/icfpcontest2020/dockerfiles/master/dockerfiles/rust/Dockerfile'
      }
    }
    stage('Build image') {
      agent { dockerfile true }
      steps {
        echo 'Hello world!'
      }
    }
  }
}