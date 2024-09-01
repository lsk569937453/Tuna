import { Button, Card, Pagination } from "flowbite-react";
import { Table, Modal } from "flowbite-react";
import { HiOutlinePlus, HiSearch, HiShoppingCart } from "react-icons/hi";
import ReactECharts from 'echarts-for-react';
import Request from "./utils/axiosUtils";
import { useEffect, useState } from "react"; import BatteryGauge from 'react-battery-gauge'
import moment from 'moment';
import 'react-toastify/dist/ReactToastify.css';

import { ToastContainer, toast } from 'react-toastify';

const pageSize = 5;

function DatasourcePage() {

    const [openModal, setOpenModal] = useState(false);
    const [datasourceName, setDatasourceName] = useState("");
    const [datasourceUrl, setDatasourceUrl] = useState("");
    //0代表插入，1代表更新
    const [modalType, setModalType] = useState(0);
    const [datasourceTableData, setDatasourceTableData] = useState([]);
    useEffect(() => {
        getDatasourceData();
    }, []);


    const getDatasourceData = () => {
        Request.get("/api/datasource").then((res) => {
            console.log(res);
            const mesArray = res.data.message.map(
                ({
                    id: id,
                    datasource_name: datasourceName,
                    datasource_url: datasourceUrl,
                    addr: addr,
                    timestamp: timestamp

                }) => {
                    return {
                        id,
                        datasourceName,
                        datasourceUrl,
                        addr,
                        timestamp
                    };
                }
            );
            setDatasourceTableData(mesArray);
        });
    };
    const deleteDatasource = (id) => {
        Request.delete("/api/datasource/" + id).then((res) => {
            console.log(res);
            if (res.data.resCode != 0) {
                // this.$message.error('添加错误:' + res.data.message);
                toast.error("删除任务出错!", {
                    position: "top-center"
                });
            } else {
            }
            window.location.reload();

        })
    }
    const addDatasource = () => {
        Request.post("/api/datasource", {
            "datasource_name": datasourceName,
            "datasource_url": datasourceUrl,

        }).then((res) => {

            console.log(res);
            if (res.data.resCode != 0) {
                // this.$message.error('添加错误:' + res.data.message);
                toast.error("添加任务出错!", {
                    position: "top-center"
                });
            } else {
                // this.addUserDialogVisible = false;

                // this.form.id = 0;
                // this.form.name = "";
                // this.$router.push("/refresh")
                toast.info("添加任务成功!", {
                    position: "top-center"
                });
                window.location.reload();
            }
        })
            .catch(err => {
                console.log(err)
            });
    }
    return (
        <div className="flex flex-col">

            <div className="p-4 flex-col">
                <div className="mb-4 flex justify-center">

                    <Button onClick={() => setOpenModal(true)}>添加数据源</Button>
                    <ToastContainer />

                    <Modal dismissible show={openModal} onClose={() => setOpenModal(false)} >
                        <div className="flex flex-col items-center gap-4 p-5 ">
                            <div className="flex items-center w-full">
                                <span className="mr-2 basis-1/3 text-right	">数据源名称:</span>
                                <input type="text" className="basis-1/3 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block  p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="数据源名称" required
                                    onChange={(e) => setDatasourceName(e.target.value)}
                                    value={datasourceName}
                                />
                            </div>
                            <div className="flex items-center   w-full">
                                <span className="mr-2 basis-1/3 text-right	">数据源地址:</span>
                                <input type="text" className="basis-1/3 bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block  p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="数据源地址:" required
                                    onChange={(e) => setDatasourceUrl(e.target.value)}
                                    value={datasourceUrl}

                                />
                            </div>

                            <div className="flex items-center  w-full">
                                <div className="mr-2  basis-1/3">
                                </div>
                                {modalType == 0 &&
                                    <Button className="basis-1/3" onClick={addDatasource}>添加</Button>}
                                {modalType == 1 &&
                                    <Button className="basis-1/3" onClick={confirmEditFurnace}>更新</Button>}
                            </div>
                        </div>

                    </Modal>
                </div>
                <Table>
                    <Table.Head>
                        <Table.HeadCell className="font-bold text-center text-xl">数据源名称</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">数据源地址</Table.HeadCell>
                        <Table.HeadCell className="font-bold text-center text-xl">操作</Table.HeadCell>
                    </Table.Head>
                    <Table.Body className="divide-y">
                        {datasourceTableData.map((row, index) => (
                            <Table.Row className="bg-white dark:border-gray-700 dark:bg-gray-800" key={index}>
                                <Table.Cell className="whitespace-nowrap font-medium text-gray-900 dark:text-white text-center">
                                    {row.datasourceName}
                                </Table.Cell>
                                <Table.Cell className="text-center">  {row.datasourceUrl}</Table.Cell>

                                <Table.Cell className="text-center">
                                    <a href="#" className="font-medium text-cyan-600 hover:underline dark:text-cyan-500"
                                        onClick={() => deleteDatasource(row.id)}>
                                        删除
                                    </a>
                                </Table.Cell>

                            </Table.Row>
                        ))}

                    </Table.Body>
                </Table>
            </div>
        </div >
    );
}
export default DatasourcePage;