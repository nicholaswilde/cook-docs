package document

import(
  //"os"
  "text/template"
  "github.com/Masterminds/sprig/v3"
  //log "github.com/sirupsen/logrus"
)

func newRecipeDocumentationTemplate(recipe string, recipePath string) (*template.Template) {
  t := template.New(recipePath)
  t.Funcs(sprig.TxtFuncMap())
  t.Parse(recipe)
  return t
}
