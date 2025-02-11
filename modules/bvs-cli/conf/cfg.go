package conf

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/viper"
)

var C *Conf

func InitConfig() {
	configPath := os.Getenv("SATLAYER_CONFIG")
	if configPath == "" {
		// get home directory
		home := checkConfig()
		viper.SetConfigName("config")                                   // get config file
		viper.SetConfigType("toml")                                     // file type
		viper.AddConfigPath(filepath.Join(home, ".config", "satlayer")) // config file path
	} else {
		viper.SetConfigFile(configPath)
	}

	// read config file
	if err := viper.ReadInConfig(); err != nil {
		fmt.Println("Error reading config file:", err)
	}
	if err := viper.Unmarshal(&C); err != nil {
		panic(fmt.Sprintf("config file invalid. %+x", err))
	}
}

func checkConfig() string {
	home, err := os.UserHomeDir()
	if err != nil {
		panic(fmt.Sprintf("Error getting home directory: %s\n", err))
	}
	configDir := filepath.Join(home, ".config", "satlayer")
	configFile := filepath.Join(configDir, "config.toml")

	if _, err := os.Stat(configDir); os.IsNotExist(err) {
		if err := os.MkdirAll(configDir, os.ModePerm); err != nil {
			panic(fmt.Sprintf("Error creating directory: %s\n", err))
		}
	}

	if _, err := os.Stat(configFile); os.IsNotExist(err) {
		file, err := os.Create(configFile)
		if err != nil {
			panic(fmt.Sprintf("Error creating file: %s\n", err))
		}
		defer file.Close()
		config := strings.Replace(content, "{keyDir}", home+"/.babylond", 1)
		if _, err = file.WriteString(config); err != nil {
			panic(fmt.Sprintf("Error writing to file: %s\n", err))
		}
	}
	return home
}
