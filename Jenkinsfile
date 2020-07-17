pipeline {
  agent any;

  environment {
    GITHUB_TOKEN = credentials('GitHubToken')
    GITHUB_URL_PREFIX = 'https://api.github.com/repos/Skobochka/icfpc-2020/statuses'
    DOCKERFILE_URL = 'https://raw.githubusercontent.com/icfpcontest2020/dockerfiles/master/dockerfiles/rust/Dockerfile'
  }

  stages {
    stage('Fetch Dockerfile') {
      steps {
        sh "wget -O Dockerfile ${DOCKERFILE_URL}"
      }
    }

    stage('Build') {
      steps {
        timeout(time: 10, unit: 'MINUTES') {
          ansiColor('xterm') {
            sh "docker build -t icfpc2020-rust-org-image:${env.BUILD_TAG} --network=none ."
          }
        }
      }
    }

    // DISABLED until proper server found
    // stage('Smoke Test') {
    //   steps {
    //     sh "docker run -t --rm -e RUST_BACKTRACE=1 icfpc2020-rust-org-image:${env.BUILD_TAG} http://server:12345 2933935384595749692"
    //   }
    // }

    stage('Test') {
      steps {
        ansiColor('xterm') {
          sh "docker run -t --rm --network=none -e RUST_BACKTRACE=1 --entrypoint ./test.sh icfpc2020-rust-org-image:${env.BUILD_TAG}"
        }
      }
    }
  }
  post { 
    cleanup {
      sh "docker rmi icfpc2020-rust-org-image:${env.BUILD_TAG} || true" // Do not signal error if no image found
    }
    success {
      script {
        def actions = []
        for (int i = 0; i < currentBuild.changeSets.size(); i++) {
          def entries = currentBuild.changeSets[i].items
          for (int j = 0; j < entries.length; j++) {
            def entry = entries[j]
            actions << """
               set +x
               curl -s "$GITHUB_URL_PREFIX/${entry.commitId}" \
                 -H "Authorization: token $GITHUB_TOKEN" \
                 -H "Content-Type: application/json" \
                 -X POST \
                 -d \"{\\\"state\\\": \\\"success\\\", \\\"context\\\": \\\"continuous-integration/jenkins\\\", \\\"description\\\": \\\"Jenkins\\\", \\\"target_url\\\": \\\"$BUILD_URL/console\\\"}\" > /dev/null
              """
          }
        }

        for (int i = 0; i < actions.size(); i++) {
          sh actions[i]
        }
      }
    }
    failure {
      script {
        def actions = []
        for (int i = 0; i < currentBuild.changeSets.size(); i++) {
          def entries = currentBuild.changeSets[i].items
          for (int j = 0; j < entries.length; j++) {
            def entry = entries[j]
            actions << """
               set +x
               curl "$GITHUB_URL_PREFIX/${entry.commitId}" \
                 -H "Authorization: token $GITHUB_TOKEN" \
                 -H "Content-Type: application/json" \
                 -X POST \
                 -d \"{\\\"state\\\": \\\"failure\\\", \\\"context\\\": \\\"continuous-integration/jenkins\\\", \\\"description\\\": \\\"Jenkins\\\", \\\"target_url\\\": \\\"$BUILD_URL/console\\\"}\"
              """
          }
        }

        for (int i = 0; i < actions.size(); i++) {
          sh actions[i]
        }
      }
    }
  }
}