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
	"reflect"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/chelnak/ysmrr"
	"github.com/chelnak/ysmrr/pkg/animations"
	"github.com/chelnak/ysmrr/pkg/colors"
	"github.com/ttacon/chalk"
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

func catchInterrupts() {
	c := make(chan os.Signal)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-c
		cleanup()
		os.Exit(1)
	}()
}

type Config struct {
	Location string
}

func getConfig() (config Config) {
	reader := bufio.NewReader(os.Stdin)
	fmt.Println("Storage location to import VMs (e.g. local, local-lvm)")
	fmt.Print("Location: ")
	input, _ := reader.ReadString('\n')
	location := strings.TrimSuffix(input, "\n")

	config = Config{
		Location: location,
	}

	return
}

var images = []Template{
	{
		VMID: 1011,
		Name: "Ubuntu 18.04",
		Link: "https://cdn.convoypanel.com/ubuntu/ubuntu-18-04-amd64.vma.zst",
	},
	{
		VMID: 1000,
		Name: "Ubuntu 20.04",
		Link: "https://cdn.convoypanel.com/ubuntu/ubuntu-20-04-amd64.vma.zst",
	},
	{
		VMID: 1001,
		Name: "Ubuntu 22.04",
		Link: "https://cdn.convoypanel.com/ubuntu/ubuntu-22-04-amd64.vma.zst",
	},
	{
		VMID: 1002,
		Name: "Windows Server 2022",
		Link: "https://cdn.convoypanel.com/windows/windows-2022-datacenter-amd64.vma.zst",
	},
	{
		VMID: 1003,
		Name: "Windows Server 2019",
		Link: "https://cdn.convoypanel.com/windows/windows-2019-datacenter-amd64.vma.zst",
	},
	{
		VMID: 1004,
		Name: "AlmaLinux 8",
		Link: "https://cdn.convoypanel.com/almalinux/almalinux-8-amd64.vma.zst",
	},
	{
		VMID: 1005,
		Name: "AlmaLinux 9",
		Link: "https://cdn.convoypanel.com/almalinux/almalinux-9-amd64.vma.zst",
	},
	{
		VMID: 1006,
		Name: "Centos 7",
		Link: "https://cdn.convoypanel.com/centos/centos-7-amd64.vma.zst",
	},
	{
		VMID: 1007,
		Name: "Centos 8",
		Link: "https://cdn.convoypanel.com/centos/centos-8-amd64.vma.zst",
	},
	{
		VMID: 1008,
		Name: "Debian 11",
		Link: "https://cdn.convoypanel.com/debian/debian-11-amd64.vma.zst",
	},
	{
		VMID: 1009,
		Name: "Rocky Linux 8",
		Link: "https://cdn.convoypanel.com/rocky-linux/rocky-linux-8-amd64.vma.zst",
	},
	{
		VMID: 1010,
		Name: "Rocky Linux 9",
		Link: "https://cdn.convoypanel.com/rocky-linux/rocky-linux-9-amd64.vma.zst",
	},
}

func ChanToSlice(ch interface{}) interface{} {
	chv := reflect.ValueOf(ch)
	slv := reflect.MakeSlice(reflect.SliceOf(reflect.TypeOf(ch).Elem()), 0, 0)
	for {
		v, ok := chv.Recv()
		if !ok {
			return slv.Interface()
		}
		slv = reflect.Append(slv, v)
	}
}

func getImages(wg *sync.WaitGroup) []Template {
	p := mpb.New(mpb.WithWaitGroup(wg))

	var (
		mu        = &sync.Mutex{}
		destSlice = make([]Template, 0)
	)

	for _, image := range images {
		fileName := "vzdump-qemu-" + strconv.Itoa(image.VMID) + ".vma.zst"
		wg.Add(1)

		go func(image Template) {
			if err := exec.Command("bash", "-c", fmt.Sprintf("qm status %d", image.VMID)).Run(); err == nil {
				fmt.Printf("Image '%s' already exists!\n", image.Name)

				wg.Done()
				return
			}

			if _, err := os.Stat(fileName); err != nil {
				DownloadFile(image.Link, "./", fileName, image.Name, p)
			} else {
				fmt.Printf("Image '%s' is already downloaded!\n", image.Name)
			}

			mu.Lock()
			destSlice = append(destSlice, image)
			mu.Unlock()

			defer wg.Done()
		}(image)
	}

	p.Wait()

	return destSlice
}

type Spinner struct {
	Spinner *ysmrr.Spinner
}

func main() {
	catchInterrupts()

	config := getConfig()

	var wg sync.WaitGroup

	downloaded := getImages(&wg)

	fmt.Printf("Importing VMs to %s\n", config.Location)

	manager := ysmrr.NewSpinnerManager(
		ysmrr.WithAnimation(animations.Pipe),
		ysmrr.WithSpinnerColor(colors.FgHiBlue),
	)

	spinners := make([]Spinner, 0)

	// add spinners
	for _, image := range downloaded {
		s := manager.AddSpinner(fmt.Sprintf("Importing %s (vmid: %d)\n", image.Name, image.VMID))

		spinners = append(spinners, Spinner{s})
	}

	wg.Add(len(spinners))
	manager.Start()

	for spinnerIndex, image := range downloaded {

		go func(image Template, index int, spinners []Spinner) {
			s := spinners[index].Spinner

			time.Sleep(time.Second)

			err := exec.Command("bash", "-c", fmt.Sprintf("qmrestore vzdump-qemu-%d.vma.zst %d -storage %s", image.VMID, image.VMID, config.Location)).Run()

			if err != nil {
				s.Error()
				s.UpdateMessage(fmt.Sprintf("Failed to import %s (vmid: %d)\n", image.Name, image.VMID))
				wg.Done()
				return
			}

			s.UpdateMessage(fmt.Sprintf("Imported %s (vmid: %d)\n", image.Name, image.VMID))
			s.Complete()
			wg.Done()
		}(image, spinnerIndex, spinners)
	}

	wg.Wait()
	manager.Stop()

	fmt.Println(chalk.White.NewStyle().WithBackground(chalk.Green), "Images locked and loaded. Start capitalizing on servers!", chalk.Reset)
	cleanup()
}
