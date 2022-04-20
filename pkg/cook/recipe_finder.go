package cook

import (
	"os"
	"path/filepath"

	log "github.com/sirupsen/logrus"
	"github.com/spf13/viper"
	"github.com/nicholaswilde/cook-docs/pkg/util"
)

func FindRecipePaths(recipeSearchRoot string) ([]string, error) {
	ignoreFilename := viper.GetString("ignore-file")
	ignoreContext := util.NewIgnoreContext(ignoreFilename)
	recipePaths := make([]string, 0)

	err := filepath.Walk(recipeSearchRoot, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if info.IsDir() && ignoreContext.ShouldIgnore(path, info) {
			log.Debugf("Ignoring directory %s", path)
			return filepath.SkipDir
		}

		if filepath.Ext(path) == ".cook" {
			if ignoreContext.ShouldIgnore(path, info) {
				log.Debugf("Ignoring recipe file %s", path)
				return nil
			}
			recipePaths = append(recipePaths, path)
		}
		return nil
	})

	return recipePaths, err
}
