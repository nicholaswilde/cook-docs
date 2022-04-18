package cook

import (
	"os"
	"path/filepath"

	log "github.com/sirupsen/logrus"
	"github.com/spf13/viper"
	"github.com/nicholaswilde/cook-docs/pkg/util"
)

func FindRecipeDirectories(recipeSearchRoot string) ([]string, error) {
	ignoreFilename := viper.GetString("ignore-file")
	ignoreContext := util.NewIgnoreContext(ignoreFilename)
	recipeDirs := make([]string, 0)

	err := filepath.Walk(recipeSearchRoot, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		absolutePath, _ := filepath.Abs(path)

		if info.IsDir() && ignoreContext.ShouldIgnore(absolutePath, info) {
			log.Debugf("Ignoring directory %s", path)
			return filepath.SkipDir
		}

		if ignoreContext.ShouldIgnore(absolutePath, info) {
			log.Debugf("Ignoring recipe file %s", path)
			return nil
		}

		if filepath.Ext(path) == ".cook" {
			recipeDirs = append(recipeDirs, path)
		}
		return nil
	})

	return recipeDirs, err
}
