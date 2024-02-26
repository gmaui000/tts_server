import os
import re
# import sys
import shutil


# sys.setrecursionlimit(3000)
# 头文件目录列表
include_dirs = ['tensorflow/', 'tensorflow/tensorflow/']

# 存储所有头文件及其依赖
header_files = set()

# 递归查找源文件依赖的头文件


def find_headers(source_file):
    # 打开源文件
    with open(source_file, 'r') as f:
        for line in f:
            # 忽略注释
            if '//' in line:
                line = line[:line.index('//')]
            if '/*' in line:
                line = line[:line.index('/*')]
            if '*/' in line:
                line = line[line.index('*/') + 2:]
            line = line.strip()

            # 匹配 #include 行
            match = re.match(r'#include\s+\"(.*)\"', line)
            if match:
                # 获取头文件路径
                header = match.group(1)

                # 忽略 <> 引入的头文件
                if '<' in header:
                    continue

                # 添加头文件路径
                # header_files.add(header)

                # 递归查找头文件依赖
                found = False
                for dir in include_dirs:
                    header_path = os.path.join(dir, header)
                    if header_path in header_files:
                        found = True
                        break
                    if os.path.exists(header_path):
                        found = True
                        # print(header_path)
                        header_files.add(header_path)
                        find_headers(header_path)

                if (not found):
                    # print("Found path: " + header_path)
                    # else:
                    print("Not found path: " + header_path)

# 查找所有源文件依赖的头文件


def find_all_headers():
    headers = [
        'tensorflow/tensorflow/lite/interpreter.h',
        'tensorflow/tensorflow/lite/optional_debug_tools.h',
        'tensorflow/tensorflow/lite/model.h',
        'tensorflow/tensorflow/lite/kernels/register.h',
        'tensorflow/tensorflow/lite/c/common.h' ]

    # 遍历头文件目录
    # for dir in include_dirs:
    #    for root, dirs, files in os.walk(dir):
    #        for file in files:
    #            if file.endswith('.h') or file.endswith('.hpp'):
    #                header_path = os.path.join(root, file)
    #                header_files.add(header_path)

    for header in headers:
        header_files.add(header)
        find_headers(header)

    return header_files


# 测试
headers = find_all_headers()
headers = list(set(headers))
for header in headers:
    # print(header)
    new_header = header.replace("tensorflow", "tf", 1)
    if (new_header != header):
        os.makedirs(os.path.dirname(new_header), exist_ok=True)
        shutil.copy(header, new_header)
