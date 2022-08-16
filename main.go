package main

import (
	"bufio"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"syscall"

	"github.com/vbauerster/mpb/v7"
	"github.com/vbauerster/mpb/v7/decor"
)

func DownloadFile(url string, dest string, name string, progressName string, p *mpb.Progress) {
	fileName := name //path.Base(url)

	filePath := fmt.Sprintf("%s/%s.tmp", dest, fileName)
	file, err := os.Create(filePath)

	if err != nil {
		fmt.Println(filePath)
		panic(err)
	}

	defer file.Close()

	resp, err := http.Get(url)

	if err != nil {
		panic(err)
	}

	defer resp.Body.Close()

	bar := p.New(resp.ContentLength, mpb.BarStyle().Rbound("|"),
		mpb.PrependDecorators(
			decor.Name(progressName+"  "),
			decor.CountersKibiByte("% .2f / % .2f"),
		),
		mpb.AppendDecorators(
			decor.EwmaETA(decor.ET_STYLE_GO, 90),
			decor.Name(" ] "),
			decor.EwmaSpeed(decor.UnitKiB, "% .2f", 60),
		))

	proxyReader := bar.ProxyReader(resp.Body)

	_, err = io.Copy(file, proxyReader)

	if err != nil {
		panic(err)
	}

	os.Rename(filePath, strings.Replace(filePath, ".tmp", "", -1))
}

func cleanup() {
	files, err := filepath.Glob("*.vma*")
	if err != nil {
		fmt.Println(err)
	}

	for _, file := range files {
		if err := os.Remove(file); err != nil {
			fmt.Println(err)
		}
	}
}

type Template struct {
	VMID int
	Name string
	Link string `json:"-"`
	Disk string `json:"-"`
}

func main() {
	c := make(chan os.Signal)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-c
		cleanup()
		os.Exit(1)
	}()

	reader := bufio.NewReader(os.Stdin)
	fmt.Println("Storage location to import VMs (e.g. local, local-lvm)")
	fmt.Print("Location: ")
	input, _ := reader.ReadString('\n')
	location := strings.TrimSuffix(input, "\n")

	images := []Template{
		{
			VMID: 1000,
			Name: "Ubuntu 20.04",
			Link: "https://cdn.convoypanel.com/ubuntu-20-04.vma.zst",
		},
	}

	var wg sync.WaitGroup
	p := mpb.New(mpb.WithWaitGroup(&wg))

	for _, image := range images {
		fileName := strconv.Itoa(image.VMID) + ".vma.zst"
		wg.Add(1)

		go func(image Template) {
			if _, err := os.Stat(fileName); err != nil {
				DownloadFile(image.Link, "./", fileName, image.Name, p)
			} else {
				fmt.Printf("Image '%s' already exists!\n", image.Name)
			}

			wg.Done()
		}(image)
	}

	p.Wait()

	fmt.Printf("Importing VMs to %s", location)

	for _, image := range images {
		wg.Add(1)

		go func(image Template) {
			fmt.Printf("Importing %s (vmid: %d)\n", image.Name, image.VMID)

			defer wg.Done()

			fmt.Println(fmt.Sprintf("qmrestore %d.vma.zst %d -storage %s", image.VMID, image.VMID, location))

			err := exec.Command("bash", "-c", fmt.Sprintf("qmrestore %d.vma.zst %d -storage %s", image.VMID, image.VMID, location)).Run()
			if err != nil {
				panic(err)
			}

			fmt.Printf("Imported %s (vmid: %d)\n", image.Name, image.VMID)
		}(image)
	}

	wg.Wait()

	fmt.Println("Cleaning up...")
	cleanup()
}
