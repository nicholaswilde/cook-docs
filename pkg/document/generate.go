package document

import (
  //"fmt"
  //"encoding/json"
  "text/template"
  "os"
  "github.com/aquilax/cooklang-go"
	log "github.com/sirupsen/logrus"
)

func PrintDocumentation(recipePath string, r *cooklang.Recipe) {
  log.Infof("Generating README Documentation for recipe %s", recipePath)
  //j, err := json.MarshalIndent(r, "", "  ")
  //if err != nil {
  //  log.Fatal(err)
  //}
  //fmt.Println(string(j))
  const letter = `
{{.Metadata.servings}}
`
  t := template.Must(template.New("letter").Parse(letter))
  err := t.Execute(os.Stdout, r)
  if err != nil {
    log.Println("executing template:", err)
  }
}
